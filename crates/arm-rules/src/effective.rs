//! Effective scores and score limits.
//!
//! Two kinds of effect feed this module, both data-driven (each virtue/flaw
//! declares [`Effect`]s naming their target via a selection's parameter value;
//! the engine hardcodes no IDs):
//!
//! - *Ability bonuses* (Puissant Ability +2) add to a bought ability score; the
//!   effective ability score is bought + bonus, always computed, never stored.
//! - *Characteristic limit shifts* (Great/Poor Characteristic) move a base
//!   score's buy cap or floor. They grant no points — the score is still bought
//!   against the cost table — so there is no characteristic "effective score";
//!   instead [`characteristic_cap`] / [`characteristic_floor`] report the
//!   per-characteristic range the limit shifts open.
//!
//! Source: `Ars Magica - Definitive Edition (Core Rules).md:4814-4816` (Puissant
//! Ability, +2), `:3987-3989` (Great Characteristic, raise to +5), `:6598-6600`
//! (Poor Characteristic, lower to −5).

use crate::ability::AbilityCategory;
use crate::characteristics::Characteristic;
use crate::grant::{Grant, GrantConstraint, resolve_grants};
use crate::ruleset::Ruleset;
use crate::types::{
    AgingEffect, Effect, Entity, EntityTypeProfile, Id, ItemKind, Magnitude, MightScore, Realm,
    ReputationType, Selection, SpellSelection,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

mod ability;
pub use ability::*;
mod art;
pub use art::*;
mod characteristic;
pub use characteristic::*;
mod xp;
pub use xp::*;
mod spell;
pub use spell::*;
mod gift_confidence;
pub use gift_confidence::*;
mod might;
pub use might::*;
mod warping;
pub use warping::*;
mod reputation_and_caps;
pub use reputation_and_caps::*;

/// The selection list every effect / score computation iterates: the entity's
/// bought selections plus any Virtue rows its Hermetic House grants (see
/// [`crate::house::granted_selections`]). Borrows `entity.selections` untouched
/// when the House grants nothing — the common case (any non-magus, or a magus
/// whose House has no resolved grant), so no allocation. Otherwise returns the
/// concatenation `bought ++ granted`.
///
/// Balance and caps deliberately do **not** route through this — they stay on
/// `entity.selections` so House grants are free of the point budget and exempt
/// from the count caps (a granted Major Hermetic Virtue cannot trip the
/// `≤1 Major Hermetic Virtue` cap).
pub(crate) fn selections_for_effects<'a>(
    entity: &'a Entity,
    ruleset: &Ruleset,
) -> Cow<'a, [Selection]> {
    let granted = entity_grants(entity, ruleset);
    if granted.is_empty() {
        Cow::Borrowed(&entity.selections)
    } else {
        let mut combined = entity.selections.clone();
        combined.extend(granted);
        Cow::Owned(combined)
    }
}

/// All free-Virtue [`Selection`] rows the entity's type-linked profiles grant:
/// its Hermetic House (magi) plus its Mythic Companion type (mythic companions),
/// in that order. A character is a magus **or** a mythic companion, never both,
/// so in practice at most one source contributes — but unioning both is correct
/// and keeps the single grant-fold path uniform. The single entry point every
/// grant consumer (effects, prerequisites, the frontend's read-only granted
/// rows) uses so House and mythic grants are always treated identically.
pub fn entity_grants(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    let mut granted = entity_grants_base(entity, ruleset);
    granted.extend(warping_granted_selections(entity, ruleset));
    granted
}

/// The grant rows that feed the Warping Score which DECIDES how many warping V/F
/// are owed: House + Mythic + `grants_selection` grants, but **not** the owed
/// warping fills themselves. Keeping the owed fills out of this list is the
/// recursion guard — a fill that carries [`Effect::WarpingGrant`] cannot raise
/// the score that determines how many fills are owed (see [`warping_owed`]).
fn entity_grants_base(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    let mut granted = crate::house::granted_selections(entity, ruleset);
    granted.extend(crate::mythic_companion::granted_selections(entity, ruleset));
    granted.extend(vf_granted_selections(entity, ruleset));
    granted
}

/// Free Virtue/Flaw [`Selection`] rows granted by an [`Effect::GrantsSelection`]
/// on a bought selection (Templar Commander → Brother-Knight + Temporal
/// Influence). Scans `entity.selections` (bought only) so a granted item's own
/// `grants_selection` is not applied recursively — one level of nesting. The
/// rows are budget-exempt, exactly like House grants.
fn vf_granted_selections(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    let mut out = Vec::new();
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::GrantsSelection { items } = effect {
                out.extend(items.iter().cloned().map(Selection::new));
            }
        }
    }
    out
}

/// The [`Effect`] variants that never contribute to a score-space bonus, a
/// characteristic-limit shift, or an Affinity cost reduction — the fixed
/// "everything else is a no-op" tail every fold in this section needs, because
/// the match must stay exhaustive (a new `Effect` variant is a compile error
/// here, not a silently-ignored bonus/shift/reduction). Defined once so
/// [`ability_bonus`], [`art_bonus`], `characteristic_limit_shift`,
/// [`ability_affinity`] and `art_affinity` — five folds that each need this same
/// ~40-variant list — do not hand-maintain five near-identical copies of it.
///
/// Only sound for a fold whose "interesting" arm(s) are **guarded** (`if ...`):
/// a guard can fail, so the variant must also appear here to catch that case,
/// which is why every variant any of the five call sites treats as interesting
/// is still listed. An unconditional (unguarded) interesting arm must NOT reuse
/// this macro — the variant would then be matched twice (once unconditionally,
/// once again inside this list) and `rustc`'s `unreachable_patterns` lint would
/// turn `cargo clippy -D warnings` into a build failure. `spell_levels_bonus`,
/// `general_xp_bonus`, and `spell_mastery_advancement_affinity` each match their
/// one interesting variant unconditionally, so each keeps its own shorter,
/// hand-written tail (excluding just that one variant) instead.
macro_rules! irrelevant_effect_variants {
    () => {
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicLimit { .. }
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
        // M5/5b in-play effects: consumed by derived.rs (5i); they never affect a
        // creation-legality score bonus, characteristic-limit shift, or Affinity
        // cost reduction, so they are no-ops in every fold that shares this tail.
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
        // Elemental Magic is an XP-space Art boost applied in
        // effective_art_score, not a flat bonus, shift, or Affinity reduction;
        // no-op in every fold that shares this tail.
        | Effect::ElementalMagic { .. }
    };
}
// Re-exported (rather than left textually scoped) so the domain submodules
// below can invoke it by name via their `use super::*;` — a `macro_rules!`
// item follows normal item privacy, but only a path-based `use` makes it
// resolvable from a module that isn't textually after this point in the same
// file.
pub(crate) use irrelevant_effect_variants;

/// Clamps a signed budget total to a non-negative `u32` (a net-negative grant
/// floors at 0 rather than underflowing).
fn clamp_to_u32(n: i64) -> u32 {
    u32::try_from(n.max(0)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AbilityScore, ArtScore, EntityKind, RulesetRef, Selection, SupernaturalPower,
    };
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    /// A ruleset with Puissant Ability (+2 ability), Great Characteristic (raises
    /// the buy cap, up to twice), Poor Characteristic (lowers the buy floor, up to
    /// twice), and the characteristic table extended to ±5 with base limits ±3 and
    /// effective limits ±5.
    fn ruleset() -> Ruleset {
        let items = r#"[
          {
            "id": "virtue.the_gift",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "free",
            "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.puissant_ability",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }]
          },
          {
            "id": "virtue.great_characteristic",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }],
            "effects": [{ "type": "characteristic_limit", "param": "characteristic", "amount": 1 }],
            "max_per_target": 2
          },
          {
            "id": "flaw.poor_characteristic",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }],
            "effects": [{ "type": "characteristic_limit", "param": "characteristic", "amount": -1 }],
            "max_per_target": 2
          },
          {
            "id": "virtue.puissant_art",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }]
          },
          {
            "id": "virtue.skilled_parens",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "effects": [
              { "type": "spell_levels", "amount": 30 },
              { "type": "general_xp", "amount": 60 }
            ]
          },
          {
            "id": "flaw.weak_parens",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "effects": [
              { "type": "spell_levels", "amount": -30 },
              { "type": "general_xp", "amount": -60 }
            ]
          },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          {
            "id": "companion",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"],
            "forbidden_categories": [],
            "required_traits": [],
            "forbidden_traits": [],
            "gift_policy": "forbidden",
            "gift_id": "virtue.the_gift",
            "gift_categories": [],
            "spell_levels": 120,
            "creation_phases": ["concept"]
          }
        ]"#;
        let abilities = r#"{ "abilities": [
          { "id": "ability.awareness", "category": "general" },
          { "id": "ability.stealth", "category": "general" },
          { "id": "ability.area_lore", "category": "general", "parameter": "area" }
        ] }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let characteristics = r#"{
          "start_points": 7,
          "base_max": 3, "base_min": -3,
          "effective_max": 5, "effective_min": -5,
          "costs": [
            { "score": 5, "cost": 15 }, { "score": 4, "cost": 10 },
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 },
            { "score": 1, "cost": 1 }, { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 },
            { "score": -3, "cost": -6 }, { "score": -4, "cost": -10 },
            { "score": -5, "cost": -15 }
          ]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            characteristics: Some(characteristics),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn entity(selections: Vec<Selection>) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        e.selections = selections;
        e
    }

    /// Skilled Parens's +60 XP folds into the general apprenticeship pool.
    #[test]
    fn skilled_parens_raises_general_xp_pool() {
        let rs = ruleset();
        let mut e = entity(vec![Selection::new(Id::new("virtue.skilled_parens"))]);
        e.xp_pool = 30;
        assert_eq!(xp_allocation(&e, &rs).general_pool, 90);
    }

    /// A GeneralXp modifier deeper than the base pool clamps at 0, not underflow.
    #[test]
    fn negative_general_xp_clamps_at_zero() {
        let rs = ruleset();
        let mut e = entity(vec![Selection::new(Id::new("flaw.weak_parens"))]);
        e.xp_pool = 10;
        assert_eq!(xp_allocation(&e, &rs).general_pool, 0);
    }

    fn puissant(ability: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new(ability))]),
        )
    }

    /// Puissant targeting one instance of a parameterized ability: the ability id
    /// plus the instance under the ability's own param key (`area`).
    fn puissant_instance(ability: &str, key: &str, value: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([
                ("ability".into(), Id::new(ability)),
                (key.into(), Id::new(value)),
            ]),
        )
    }

    fn lore(area: &str, score: u8) -> AbilityScore {
        AbilityScore {
            ability: Id::new("ability.area_lore"),
            score,
            specialty: None,
            parameter: Some(area.to_string()),
        }
    }

    fn great(characteristic: Characteristic) -> Selection {
        Selection::with_params(
            Id::new("virtue.great_characteristic"),
            BTreeMap::from([("characteristic".into(), characteristic.id())]),
        )
    }

    fn poor(characteristic: Characteristic) -> Selection {
        Selection::with_params(
            Id::new("flaw.poor_characteristic"),
            BTreeMap::from([("characteristic".into(), characteristic.id())]),
        )
    }

    #[test]
    fn puissant_ability_adds_two_to_its_target() {
        let rs = ruleset();
        let mut e = entity(vec![puissant("ability.awareness")]);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
        }];
        assert_eq!(
            ability_bonus(&e, &rs, &Id::new("ability.awareness"), None),
            2
        );
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.awareness"), None),
            5
        );
    }

    #[test]
    fn ability_bonus_zero_for_non_targeted_ability() {
        let rs = ruleset();
        let e = entity(vec![puissant("ability.awareness")]);
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.stealth"), None), 0);
        // No bought score and no matching bonus => effective 0.
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.stealth"), None),
            0
        );
    }

    #[test]
    fn puissant_for_two_abilities_each_applies_independently() {
        let rs = ruleset();
        let e = entity(vec![
            puissant("ability.awareness"),
            puissant("ability.stealth"),
        ]);
        assert_eq!(
            ability_bonus(&e, &rs, &Id::new("ability.awareness"), None),
            2
        );
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.stealth"), None), 2);
    }

    #[test]
    fn puissant_targets_one_lore_instance_only() {
        let rs = ruleset();
        let e = entity(vec![puissant_instance(
            "ability.area_lore",
            "area",
            "Brandenburg",
        )]);
        let lore = Id::new("ability.area_lore");
        // Only the named instance is boosted; other areas are untouched.
        assert_eq!(ability_bonus(&e, &rs, &lore, Some("Brandenburg")), 2);
        assert_eq!(ability_bonus(&e, &rs, &lore, Some("Berlin")), 0);
    }

    #[test]
    fn puissant_without_instance_key_matches_no_parameterized_instance() {
        let rs = ruleset();
        // A Puissant on a parameterized ability that names no instance (only the
        // bare id) boosts nothing — it dangles until an instance is chosen.
        let e = entity(vec![puissant("ability.area_lore")]);
        let lore = Id::new("ability.area_lore");
        assert_eq!(ability_bonus(&e, &rs, &lore, Some("Brandenburg")), 0);
        assert_eq!(ability_bonus(&e, &rs, &lore, None), 0);
    }

    #[test]
    fn ability_bonuses_returns_one_entry_per_targeted_instance() {
        let rs = ruleset();
        let e = {
            let mut e = entity(vec![
                puissant_instance("ability.area_lore", "area", "Brandenburg"),
                puissant_instance("ability.area_lore", "area", "Bavaria"),
            ]);
            e.ability_scores = vec![
                lore("Brandenburg", 3),
                lore("Berlin", 2),
                lore("Bavaria", 1),
            ];
            e
        };
        // Two targeted instances get +2; the untargeted Berlin row is omitted.
        assert_eq!(
            ability_bonuses(&e, &rs),
            vec![
                AbilityBonus {
                    ability: Id::new("ability.area_lore"),
                    parameter: Some("Brandenburg".into()),
                    bonus: 2,
                },
                AbilityBonus {
                    ability: Id::new("ability.area_lore"),
                    parameter: Some("Bavaria".into()),
                    bonus: 2,
                },
            ]
        );
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.area_lore"), Some("Brandenburg")),
            5
        );
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.area_lore"), Some("Berlin")),
            2
        );
    }

    #[test]
    fn default_cap_and_floor_are_the_base_limits() {
        // With no Great/Poor, every characteristic's range is the base ±3.
        let rs = ruleset();
        let e = entity(vec![]);
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 3);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -3);
    }

    #[test]
    fn great_characteristic_raises_the_cap_without_touching_the_score() {
        let rs = ruleset();
        let mut e = entity(vec![great(Characteristic::Str)]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
        // The cap opens to +4; the bought score is unchanged (no free point).
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 4);
        assert_eq!(e.characteristics[&Characteristic::Str], 3);
        // The floor is untouched.
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -3);
    }

    #[test]
    fn great_characteristic_twice_raises_the_cap_to_five() {
        let rs = ruleset();
        let e = entity(vec![great(Characteristic::Str), great(Characteristic::Str)]);
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 5);
    }

    #[test]
    fn cap_is_clamped_at_the_effective_ceiling() {
        // Three Greats (a state the data forbids, but the engine must stay sound)
        // cannot push the cap past the +5 effective ceiling.
        let rs = ruleset();
        let e = entity(vec![
            great(Characteristic::Str),
            great(Characteristic::Str),
            great(Characteristic::Str),
        ]);
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 5);
    }

    #[test]
    fn poor_characteristic_lowers_the_floor_without_touching_the_score() {
        let rs = ruleset();
        let mut e = entity(vec![poor(Characteristic::Str)]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, -3)]);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -4);
        assert_eq!(e.characteristics[&Characteristic::Str], -3);
        // The cap is untouched.
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 3);
    }

    #[test]
    fn poor_characteristic_twice_lowers_the_floor_to_minus_five() {
        let rs = ruleset();
        let e = entity(vec![poor(Characteristic::Str), poor(Characteristic::Str)]);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -5);
    }

    #[test]
    fn floor_is_clamped_at_the_effective_floor() {
        let rs = ruleset();
        let e = entity(vec![
            poor(Characteristic::Str),
            poor(Characteristic::Str),
            poor(Characteristic::Str),
        ]);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -5);
    }

    #[test]
    fn limit_shift_targets_only_the_named_characteristic() {
        let rs = ruleset();
        let e = entity(vec![great(Characteristic::Str), poor(Characteristic::Qik)]);
        // Str's cap rose; its floor and the other characteristic are unaffected.
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 4);
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Qik), 3);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Qik), -4);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), -3);
    }

    #[test]
    fn cap_and_floor_maps_cover_all_eight_characteristics() {
        let rs = ruleset();
        let e = entity(vec![great(Characteristic::Str), poor(Characteristic::Qik)]);
        let caps = characteristic_caps(&e, &rs);
        let floors = characteristic_floors(&e, &rs);
        assert_eq!(caps.len(), Characteristic::ALL.len());
        assert_eq!(floors.len(), Characteristic::ALL.len());
        assert_eq!(caps[&Characteristic::Str], 4);
        assert_eq!(caps[&Characteristic::Int], 3);
        assert_eq!(floors[&Characteristic::Qik], -4);
        assert_eq!(floors[&Characteristic::Int], -3);
    }

    #[test]
    fn cap_and_floor_default_to_zero_without_characteristic_rules() {
        // A ruleset that supplies no characteristic table has no base/effective
        // limits to report, so both guards fall back to zero rather than reading
        // an absent table.
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: "[]",
            type_profiles: "[]",
            ..RulesetSources::default()
        })
        .unwrap();
        let e = entity(vec![]);
        assert_eq!(characteristic_cap(&e, &rs, Characteristic::Str), 0);
        assert_eq!(characteristic_floor(&e, &rs, Characteristic::Str), 0);
    }

    fn puissant_art(art: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".into(), Id::new(art))]),
        )
    }

    #[test]
    fn puissant_art_adds_three_to_its_target() {
        let rs = ruleset();
        let mut e = entity(vec![puissant_art("art.ignem")]);
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 5,
        }];
        assert_eq!(art_bonus(&e, &rs, &Id::new("art.ignem")), 3);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 8);
    }

    #[test]
    fn art_bonus_zero_for_non_targeted_art() {
        let rs = ruleset();
        let e = entity(vec![puissant_art("art.ignem")]);
        assert_eq!(art_bonus(&e, &rs, &Id::new("art.creo")), 0);
        // No bought score and no matching bonus => effective 0.
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.creo")), 0);
    }

    #[test]
    fn puissant_art_twice_for_one_art_stacks() {
        // Two Puissant Art selections on the same Art stack (+6). The rules forbid
        // this (take twice for two *different* Arts), but the bonus calc must stay
        // sound; the legality is a validation concern.
        let rs = ruleset();
        let e = entity(vec![puissant_art("art.ignem"), puissant_art("art.ignem")]);
        assert_eq!(art_bonus(&e, &rs, &Id::new("art.ignem")), 6);
    }

    #[test]
    fn art_bonuses_returns_one_entry_per_boosted_art() {
        let rs = ruleset();
        let mut e = entity(vec![puissant_art("art.ignem")]);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 3,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 2,
            },
        ];
        // Only Ignem is boosted; Creo has no bonus and is omitted.
        assert_eq!(
            art_bonuses(&e, &rs),
            vec![ArtBonus {
                art: Id::new("art.ignem"),
                bonus: 3,
            }]
        );
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 5);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.creo")), 3);
    }

    #[test]
    fn art_bonuses_include_puissant_art_at_bought_zero() {
        // Regression (Issue 13): a Puissant Art virtue whose target has 0 bought
        // points has no `art_scores` row, yet the flat bonus must still surface so
        // the effective badge shows before the first point is bought.
        let rs = ruleset();
        let e = entity(vec![puissant_art("art.ignem")]);
        assert!(e.art_scores.is_empty());
        assert_eq!(
            art_bonuses(&e, &rs),
            vec![ArtBonus {
                art: Id::new("art.ignem"),
                bonus: 3,
            }]
        );
    }

    #[test]
    fn ability_bonuses_still_omit_zero_entries() {
        let rs = ruleset();
        let mut e = entity(vec![puissant("ability.awareness")]);
        e.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 2,
                specialty: None,
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.stealth"),
                score: 1,
                specialty: None,
                parameter: None,
            },
        ];

        let abilities = ability_bonuses(&e, &rs);
        assert_eq!(
            abilities,
            vec![AbilityBonus {
                ability: Id::new("ability.awareness"),
                parameter: None,
                bonus: 2,
            }]
        );
    }

    // ---- Phase 3: Affinity, restricted pools, characteristic points, grants ----

    /// charged_cost is the inverse of "counts as num/den, rounded up". The book's
    /// worked example: Perdo 10 needs 55 on the Art table; with Affinity 3/2 you
    /// pay 37 (which counts as ceil(37·3/2)=56 ≥ 55). Source: Ars Magica -
    /// Definitive Edition (Core Rules).md:2443.
    #[test]
    fn affinity_charged_cost_matches_perdo_example() {
        assert_eq!(charged_cost(55, Some((3, 2))), 37); // Affinity with Art
        assert_eq!(charged_cost(55, None), 55); // no affinity: full price
        // Ability table is 5× the Art table; the same ratio applies to its costs.
        assert_eq!(charged_cost(275, Some((3, 2))), 184); // ceil(275·2/3)
        assert_eq!(charged_cost(0, Some((3, 2))), 0);
    }

    /// A ruleset for the XP-pool tests: priced Ability (5×-triangular) and Art
    /// (triangular) tables, categorized abilities, and the Phase-3 virtues.
    fn xp_ruleset() -> Ruleset {
        let items = r#"[
          {
            "id": "virtue.the_gift",
            "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.affinity_ability",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "affinity_ability_cost", "param": "ability", "counts_as_num": 3, "counts_as_den": 2 }]
          },
          {
            "id": "virtue.affinity_art",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "hermetic",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "affinity_art_cost", "param": "art", "counts_as_num": 3, "counts_as_den": 2 }]
          },
          {
            "id": "virtue.educated",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.latin", "ability.artes_liberales"] }]
          },
          {
            "id": "virtue.warrior",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["martial"] }]
          },
          {
            "id": "virtue.privileged_upbringing",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["general", "academic", "martial"] }]
          },
          {
            "id": "virtue.improved_characteristics",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "characteristic_points", "amount": 3 }]
          },
          {
            "id": "flaw.weak_characteristics",
            "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "characteristic_points", "amount": -3 }]
          },
          {
            "id": "virtue.second_sight",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }]
          },
          {
            "id": "virtue.giant_blood",
            "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general",
            "entity_kinds": ["character"],
            "effects": [
              { "type": "size_delta", "amount": 2 },
              { "type": "characteristic_score_delta", "characteristic": "characteristic.str", "amount": 1 },
              { "type": "characteristic_score_delta", "characteristic": "characteristic.sta", "amount": 1 }
            ]
          },
          {
            "id": "flaw.dwarf",
            "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "general",
            "entity_kinds": ["character"],
            "effects": [
              { "type": "size_delta", "amount": -2 },
              { "type": "characteristic_score_delta", "characteristic": "characteristic.str", "amount": -1 },
              { "type": "characteristic_score_delta", "characteristic": "characteristic.sta", "amount": -1 }
            ]
          },
          {
            "id": "flaw.warped_by_magic",
            "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [{ "type": "warping_grant", "score": 1, "points": 5 }]
          },
          {
            "id": "virtue.true_faith",
            "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "true_faith_grant", "score": 1 }]
          },
          {
            "id": "virtue.magic_items",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "item_level_budget", "amount": 25 }]
          },
          {
            "id": "virtue.granter",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "grants_selection", "items": ["virtue.second_sight"] }]
          },
          {
            "id": "virtue.mastered_spells",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "hermetic",
            "entity_kinds": ["character"],
            "effects": [{ "type": "spell_mastery_xp", "amount": 50 }]
          },
          {
            "id": "virtue.flawless_magic",
            "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "hermetic",
            "entity_kinds": ["character"],
            "effects": [{ "type": "grants_spell_mastery", "score": 1, "advancement_num": 2, "advancement_den": 1 }]
          },
          {
            "id": "virtue.linguist",
            "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "group_affinity_cost", "abilities": ["ability.living_language"], "counts_as_num": 5, "counts_as_den": 4 }]
          },
          {
            "id": "virtue.demonic_blood",
            "kind": "virtue", "classification": "creation_effect", "magnitude": "major", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [
              { "type": "might_grant", "realm": "infernal", "score": 5 },
              { "type": "power_levels", "amount": 30 }
            ]
          },
          {
            "id": "virtue.demonic_might",
            "kind": "virtue", "classification": "creation_effect", "magnitude": "minor", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [{ "type": "might_grant", "realm": "infernal", "score": 2 }]
          },
          {
            "id": "virtue.demonic_powers",
            "kind": "virtue", "classification": "creation_effect", "magnitude": "minor", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [{ "type": "power_levels", "amount": 20 }]
          },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          {
            "id": "companion",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "hermetic", "supernatural"],
            "forbidden_categories": [], "required_traits": [], "forbidden_traits": [],
            "gift_policy": "allowed", "gift_id": "virtue.the_gift",
            "gift_categories": [], "creation_phases": ["concept"]
          }
        ]"#;
        // Ability table: 5·n·(n+1)/2 (5× the Art triangular). Art table: n·(n+1)/2.
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [
            { "id": "ability.latin", "category": "academic" },
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.single_weapon", "category": "martial" },
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.second_sight", "category": "supernatural", "requires_training": true }
          ]
        }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let characteristics = r#"{
          "start_points": 7, "base_max": 3, "base_min": -3,
          "effective_max": 5, "effective_min": -5,
          "costs": [
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 },
            { "score": 1, "cost": 1 }, { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 }, { "score": -3, "cost": -6 }
          ]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            characteristics: Some(characteristics),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn xp_entity(selections: Vec<Selection>) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        e.selections = selections;
        e
    }

    fn plain(ability: &str, score: u8) -> AbilityScore {
        AbilityScore {
            ability: Id::new(ability),
            score,
            specialty: None,
            parameter: None,
        }
    }

    fn sel(id: &str) -> Selection {
        Selection::new(Id::new(id))
    }

    fn sel_param(id: &str, key: &str, value: &str) -> Selection {
        Selection::with_params(Id::new(id), BTreeMap::from([(key.into(), Id::new(value))]))
    }

    #[test]
    fn affinity_with_art_reduces_charged_xp() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel_param("virtue.affinity_art", "art", "art.creo")]);
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }]; // table 15
        let alloc = xp_allocation(&e, &rs);
        // ceil(15·2/3) = 10, not 15.
        assert_eq!(alloc.total_demand, 10);
    }

    #[test]
    fn affinity_with_ability_reduces_charged_xp_on_the_5x_table() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel_param(
            "virtue.affinity_ability",
            "ability",
            "ability.awareness",
        )]);
        e.ability_scores = vec![plain("ability.awareness", 5)]; // table 75
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 50); // ceil(75·2/3) = 50
    }

    // A crafted save can carry an unbounded `ability_scores`/`art_scores`/`spells`
    // array (deserialized straight off disk, no length cap in `types.rs`). Each
    // entry becomes one `Spend`, and the flow solve allocates a dense
    // `n x n` `u32` matrix where `n` grows 1:1 with that count — a few thousand
    // entries already force a multi-hundred-MB allocation, and a real attack
    // payload (tens of thousands of entries) forces multiple GB, aborting the
    // process on a plain File -> Open with no dialog and no diagnostic
    // (CWE-400/789). `xp_allocation` must refuse a pathological entity instead
    // of building the matrix at all.
    #[test]
    #[should_panic(expected = "exceed the safety bound")]
    fn xp_allocation_refuses_a_pathological_number_of_spends() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        // Comfortably above MAX_XP_SOLVE_NODES, comfortably below anything that
        // would allocate more than a few MB if the guard did not fire first.
        e.ability_scores = vec![plain("ability.awareness", 1); 3000];
        let _ = xp_allocation(&e, &rs);
    }

    #[test]
    fn best_affinity_keeps_the_most_generous_multiplier() {
        // Affinities do not stack; the single most generous wins. A score
        // "counts as num/den of itself" is charged `table·den/num`, so a larger
        // num/den is cheaper: "counts as 2/1" (charged table·1/2) beats the
        // standard "counts as 3/2" (charged table·2/3). Order must not matter.
        assert_eq!(best_affinity([(3, 2), (2, 1)].into_iter()), Some((2, 1)));
        assert_eq!(best_affinity([(2, 1), (3, 2)].into_iter()), Some((2, 1)));
        // The chosen ratio is the one that actually charges least.
        assert_eq!(charged_cost(15, Some((2, 1))), 8); // ceil(15·1/2)
        assert_eq!(charged_cost(15, Some((3, 2))), 10); // ceil(15·2/3)
    }

    #[test]
    fn educated_pool_funds_only_its_two_abilities() {
        let rs = xp_ruleset();
        // No general pool; Educated 50 must fund Latin(15)+Artes Lib(15)=30.
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        e.ability_scores = vec![
            plain("ability.latin", 2),
            plain("ability.artes_liberales", 2),
        ];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 30);
        assert_eq!(alloc.max_flow, 30); // feasible from the restricted pool alone
        assert_eq!(alloc.restricted[0].used, 30);
    }

    #[test]
    fn granted_supernatural_first_point_is_free_second_costs_ten() {
        // A2: a Virtue-granted Supernatural Ability floor (Second Sight 1) is free
        // — the XP charge subtracts the granted floor's table cost, so the first
        // point costs 0 and score 2 costs the normal 1→2 step (15 − 5 = 10). The
        // effective score is unchanged (still the stored sheet score).
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.second_sight")]);
        e.ability_scores = vec![plain("ability.second_sight", 1)];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 0, "first granted point is free");
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
            1
        );

        e.ability_scores = vec![plain("ability.second_sight", 2)];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 10, "score 2 costs the 1→2 step only");
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
            2
        );
    }

    #[test]
    fn ungranted_supernatural_ability_still_costs_full_table() {
        // Without a granting Virtue there is no free floor: the full table applies.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.ability_scores = vec![plain("ability.second_sight", 1)];
        assert_eq!(xp_allocation(&e, &rs).total_demand, 5);
    }

    #[test]
    fn restricted_pool_is_spent_before_general_when_both_can_cover() {
        // A3: Educated's 50 restricted XP could be covered by the 100-pt general
        // pool too, but the eligible spend must drain the restricted pool first so
        // the general pool stays available and no restricted XP is wasted.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 100;
        e.ability_scores = vec![plain("ability.artes_liberales", 4)]; // 50 XP
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 50);
        assert_eq!(alloc.max_flow, 50);
        assert_eq!(alloc.general_used, 0, "general pool untouched");
        assert_eq!(alloc.restricted[0].used, 50, "restricted pool fully spent");
    }

    #[test]
    fn general_pool_covers_the_overflow_beyond_the_restricted_pool() {
        // The restricted pool is filled first; only the excess spills to general.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 100;
        e.ability_scores = vec![plain("ability.artes_liberales", 5)]; // 75 XP
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 75);
        assert_eq!(alloc.max_flow, 75);
        assert_eq!(alloc.restricted[0].used, 50, "restricted maxed out");
        assert_eq!(alloc.general_used, 25, "general covers only the overflow");
    }

    #[test]
    fn total_demand_exceeds_general_used_when_a_restricted_pool_contributes() {
        // The XP bars show "Spent" = total_demand (across ALL pools) but derive
        // "Available" from general_used only. When a restricted grant funds part of
        // the spend, general_used < total_demand, so a bare "Spent" reads as more
        // than the general pool's drop — the "more shown spent than the values
        // account for" report. This pins the invariant the UI breakdown relies on:
        // total_demand splits exactly into general_used + the restricted usage.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 100;
        e.ability_scores = vec![plain("ability.artes_liberales", 5)]; // 75 XP
        let alloc = xp_allocation(&e, &rs);
        let restricted_used: u32 = alloc.restricted.iter().map(|p| p.used).sum();
        assert!(
            alloc.general_used < alloc.total_demand,
            "restricted funding must leave general_used below the total demand",
        );
        assert_eq!(
            alloc.total_demand,
            alloc.general_used + restricted_used,
            "total demand splits exactly into general + restricted usage",
        );
    }

    #[test]
    fn total_demand_equals_general_used_without_restricted_pools() {
        // With no restricted grant, the whole spend draws on the general pool, so
        // the UI keeps the plain "Spent" label (general_used == total_demand).
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.xp_pool = 100;
        e.ability_scores = vec![plain("ability.awareness", 4)]; // 50 XP, general only
        let alloc = xp_allocation(&e, &rs);
        assert!(alloc.restricted.is_empty());
        assert_eq!(alloc.general_used, alloc.total_demand);
    }

    #[test]
    fn educated_pool_cannot_fund_an_ineligible_ability() {
        let rs = xp_ruleset();
        // Single Weapon (martial, 15) is NOT eligible for Educated; no general XP.
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        e.ability_scores = vec![plain("ability.single_weapon", 2)];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 15);
        assert_eq!(alloc.max_flow, 0); // Educated can't pay, general pool empty
    }

    /// Overlap counter-example. Latin is Academic, so eligible for BOTH Educated
    /// (narrow: 2 ids) and Privileged (broad: 3 categories); Awareness is General,
    /// eligible ONLY for Privileged. With no general XP and each pool exactly 50,
    /// the sole feasible funding is Latin→Educated and Awareness→Privileged. A
    /// greedy assignment that sent Latin (eligible everywhere) to Privileged would
    /// strand Awareness and fund only 50. The flow solver assigns globally.
    #[test]
    fn overlapping_pools_resolved_globally_not_greedily() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![
            sel("virtue.educated"),
            sel("virtue.privileged_upbringing"),
        ]);
        e.xp_pool = 0; // no general XP: the two restricted pools must cover everything
        e.ability_scores = vec![
            plain("ability.latin", 4),     // 50, academic — both pools eligible
            plain("ability.awareness", 4), // 50, general — only Privileged eligible
        ];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 100);
        assert_eq!(alloc.max_flow, 100); // fully funded only by the global assignment
        let educated = alloc
            .restricted
            .iter()
            .find(|p| p.abilities.contains(&Id::new("ability.latin")))
            .unwrap();
        let privileged = alloc
            .restricted
            .iter()
            .find(|p| p.categories.contains(&AbilityCategory::General))
            .unwrap();
        assert_eq!(educated.used, 50); // Latin had to take Educated...
        assert_eq!(privileged.used, 50); // ...leaving Privileged for Awareness
    }

    #[test]
    fn improved_characteristics_grants_three_points_stacking() {
        let rs = xp_ruleset();
        let e = xp_entity(vec![
            sel("virtue.improved_characteristics"),
            sel("virtue.improved_characteristics"),
        ]);
        assert_eq!(characteristic_points_granted(&e, &rs), 6);
    }

    #[test]
    fn linguist_group_affinity_applies_to_every_language_instance() {
        // Linguist gives a 5/4 Affinity to any Language, matched by id for every
        // instance (Ars Magica - Definitive Edition (Core Rules).md:4315-4317), unlike a param-chosen single-target Affinity.
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.linguist")]);
        let lang = Id::new("ability.living_language");
        assert_eq!(
            ability_affinity(&e, &rs, &lang, Some("German")),
            Some((5, 4))
        );
        assert_eq!(
            ability_affinity(&e, &rs, &lang, Some("Gaelic")),
            Some((5, 4))
        );
        // A non-language ability is unaffected.
        assert_eq!(
            ability_affinity(&e, &rs, &Id::new("ability.awareness"), None),
            None
        );
    }

    #[test]
    fn spell_mastery_pool_and_floor() {
        // Mastered Spells grants 50 mastery XP (stackable, Ars Magica - Definitive Edition (Core Rules).md:4471-4474);
        // Flawless Magic floors every spell's mastery at 1 (Ars Magica - Definitive Edition (Core Rules).md:3887-3889).
        let rs = xp_ruleset();
        let masters = xp_entity(vec![
            sel("virtue.mastered_spells"),
            sel("virtue.mastered_spells"),
        ]);
        assert_eq!(spell_mastery_xp(&masters, &rs), 100);
        assert_eq!(spell_mastery_floor(&masters, &rs), 0);

        let flawless = xp_entity(vec![sel("virtue.flawless_magic")]);
        assert_eq!(spell_mastery_floor(&flawless, &rs), 1);
        // A spell with no bought mastery still has effective mastery 1 under the
        // floor; a higher bought mastery wins.
        let unbought = SpellSelection {
            spell: Id::new("spell.x"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        };
        assert_eq!(effective_spell_mastery(&unbought, &flawless, &rs), 1);
        let bought = SpellSelection {
            spell: Id::new("spell.x"),
            level: None,
            mastery: Some(3),
            parameter: None,
            mastery_abilities: Vec::new(),
        };
        assert_eq!(effective_spell_mastery(&bought, &flawless, &rs), 3);
    }

    fn mastered(spell: &str, score: u8) -> SpellSelection {
        SpellSelection {
            spell: Id::new(spell),
            level: None,
            mastery: Some(score),
            parameter: None,
            mastery_abilities: Vec::new(),
        }
    }

    #[test]
    fn bought_mastery_is_charged_from_the_general_pool() {
        // Spell Mastery is an Ability bought from the Ability advancement table
        // (Ars Magica - Definitive Edition (Core Rules).md:9518, :15952). With no mastery Virtue it draws the general pool.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.xp_pool = 20;
        e.spells = vec![mastered("spell.pilum", 2)]; // table(2) = 15
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 15);
        assert_eq!(alloc.max_flow, 15, "funded from the 20-pt general pool");
        assert_eq!(alloc.general_used, 15);

        // Too small a general pool overspends by the shortfall.
        e.xp_pool = 10;
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 15);
        assert_eq!(alloc.max_flow, 10);
    }

    #[test]
    fn mastered_spells_pool_funds_mastery_but_not_abilities() {
        // Mastered Spells' +50 pool (Ars Magica - Definitive Edition (Core Rules).md:4471-4474) is spendable only on Spell
        // Mastery, never on ordinary Abilities/Arts, and the general pool is 0.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.mastered_spells")]);
        e.xp_pool = 0;
        e.spells = vec![mastered("spell.pilum", 3)]; // table(3) = 30
        e.ability_scores = vec![plain("ability.awareness", 2)]; // table(2) = 15
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 45);
        // Only the 30 mastery can be funded (from the mastery pool); the 15
        // ability spend has no pool and no general XP, so it overspends.
        assert_eq!(alloc.max_flow, 30);
        assert_eq!(alloc.general_used, 0);
    }

    #[test]
    fn restricted_ability_pool_does_not_fund_mastery() {
        // Educated's pool is Ability-only; it must not bleed into Spell Mastery.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        e.spells = vec![mastered("spell.pilum", 2)]; // table(2) = 15
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 15);
        assert_eq!(alloc.max_flow, 0, "Educated cannot fund mastery");
        // The Educated pool goes wholly unused (no eligible ability spend).
        assert_eq!(alloc.restricted[0].used, 0);
    }

    #[test]
    fn flawless_magic_floors_first_mastery_free_and_halves_the_rest() {
        // Flawless Magic auto-masters every spell at 1 (free floor) AND doubles all
        // Spell-Mastery Advancement Totals, halving the XP charged. Ars Magica - Definitive Edition (Core Rules).md:3887-3889.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.flawless_magic")]);
        e.xp_pool = 100;
        // Mastery 1 == the granted floor: free. Mastery 3: table(3) − table(1) =
        // 25, doubled advancement → ceil(25/2) = 13.
        e.spells = vec![mastered("spell.a", 1), mastered("spell.b", 3)];
        let alloc = xp_allocation(&e, &rs);
        assert_eq!(alloc.total_demand, 13);
    }

    #[test]
    fn grants_selection_folds_free_items_into_grants() {
        // A Virtue that grants another Virtue for free (Templar Commander →
        // Brother-Knight + Temporal Influence; Ars Magica - Definitive Edition (Core Rules).md:5113-5116) folds the granted
        // items into entity_grants (budget-exempt), and their effects apply.
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.granter")]);
        let grants = entity_grants(&e, &rs);
        assert!(
            grants
                .iter()
                .any(|s| s.item_ref == Id::new("virtue.second_sight")),
            "granted Second Sight should be in entity_grants: {grants:?}"
        );
        // The free-granted Second Sight seeds its ability floor at no XP cost.
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
            1
        );
    }

    #[test]
    fn item_level_budget_sums_grants() {
        // Magic Items grants +25 starting levels of enchanted devices, stackable
        // (Ars Magica - Definitive Edition (Core Rules).md:4347-4349); Redcap 50 (Ars Magica - Definitive Edition (Core Rules).md:4842-4846).
        let rs = xp_ruleset();
        assert_eq!(item_level_budget(&xp_entity(vec![]), &rs), 0);
        assert_eq!(
            item_level_budget(
                &xp_entity(vec![sel("virtue.magic_items"), sel("virtue.magic_items")]),
                &rs
            ),
            50
        );
    }

    #[test]
    fn demonic_blood_grants_infernal_might_5_and_30_power_levels() {
        // Demonic Blood confers Infernal Might (Corpus) 5 (Ars Magica 5e - Realms of Power - The Infernal.md:4120) and
        // up to 30 levels of Infernal Powers (Ars Magica 5e - Realms of Power - The Infernal.md:4122). Effective Might =
        // entity base (0 here) + Σ MightGrant of the same realm.
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.demonic_blood")]);
        let might = effective_might(&e, &rs).expect("a Demonic-Blooded being has Might");
        assert_eq!(might.realm, Realm::Infernal);
        assert_eq!(might.score, 5);
        assert_eq!(power_levels_budget(&e, &rs), 30);
    }

    #[test]
    fn demonic_might_adds_two_and_powers_add_twenty() {
        // Demonic Might: Infernal Might +2 (Ars Magica 5e - Realms of Power - The Infernal.md:4136). Demonic Powers:
        // +20 power levels (Ars Magica 5e - Realms of Power - The Infernal.md:4142). Both stack on Demonic Blood.
        let rs = xp_ruleset();
        let e = xp_entity(vec![
            sel("virtue.demonic_blood"),
            sel("virtue.demonic_might"),
            sel("virtue.demonic_powers"),
        ]);
        assert_eq!(effective_might(&e, &rs).unwrap().score, 7);
        assert_eq!(power_levels_budget(&e, &rs), 50);
    }

    #[test]
    fn effective_might_adds_grants_to_user_entered_base() {
        // A being that enters its own base Might (Strong Angelic Heritage's Divine
        // Might = age/20 is entered by hand) keeps that base; grants of the same
        // realm add on top. Powers-used sums the entered powers' levels.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.demonic_might")]);
        e.might = Some(MightScore {
            realm: Realm::Infernal,
            score: 4,
        });
        e.powers = vec![SupernaturalPower {
            name: "P".into(),
            level: 12,
        }];
        assert_eq!(effective_might(&e, &rs).unwrap().score, 6); // 4 base + 2 grant
        assert_eq!(powers_used(&e), 12);
        // No might at all → None.
        assert!(effective_might(&xp_entity(vec![]), &rs).is_none());
    }

    #[test]
    fn true_faith_grant_sums_score() {
        // True Faith grants a derived True Faith Score of 1 (Ars Magica - Definitive Edition (Core Rules).md:5169-5171),
        // base 0, summed across grants.
        let rs = xp_ruleset();
        assert_eq!(true_faith(&xp_entity(vec![]), &rs), 0);
        assert_eq!(
            true_faith(&xp_entity(vec![sel("virtue.true_faith")]), &rs),
            1
        );
    }

    #[test]
    fn warping_grant_sums_score_and_points() {
        // Warped by Magic grants 5 Warping Points; the score is DERIVED by
        // inverting the advancement curve (5 points → Warping Score 1), not read
        // from the grant's declared score. Ars Magica - Definitive Edition (Core Rules).md:7019-7021, :16464-16475.
        let rs = xp_ruleset();
        assert_eq!(
            warping(&xp_entity(vec![]), &rs),
            Warping {
                score: 0,
                points: 0
            }
        );
        assert_eq!(
            warping(&xp_entity(vec![sel("flaw.warped_by_magic")]), &rs),
            Warping {
                score: 1,
                points: 5
            }
        );
    }

    /// `warping_score`/`warping_points_total` UNIFY stored + grant-derived points
    /// through one path, then invert the advancement curve. Stored 10 + Warped by
    /// Magic's 5 = 15 points → Warping Score 2 (Ars Magica - Definitive Edition (Core Rules).md:16464-16475: cumulative
    /// 5/15/30/50/75).
    #[test]
    fn warping_sums_stored_and_granted_points_then_inverts() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("flaw.warped_by_magic")]); // +5 points
        e.warping_points = 10; // stored
        assert_eq!(warping_points_total(&e, &rs), 15);
        assert_eq!(warping_score(&e, &rs), 2);
        // `warping()` reports the unified (score, total points).
        assert_eq!(
            warping(&e, &rs),
            Warping {
                score: 2,
                points: 15
            }
        );

        // Stored points alone also invert (no grant).
        let mut only_stored = xp_entity(vec![]);
        only_stored.warping_points = 15;
        assert_eq!(warping_score(&only_stored, &rs), 2);
    }

    // --- Issue E: warping-owed V/F (Ars Magica - Definitive Edition (Core
    // Rules).md:16547-16561) --------------------------------------------------

    /// The owed-V/F threshold curve, tested on the pure `from_score` at the rule's
    /// boundary scores so the assertion is independent of the advancement table:
    /// 0 → none; 1 → 1 Minor Flaw; 3 → 2 Minor Flaws; 5 → +supernatural Minor
    /// Virtue; 6 → +1 Major Flaw; 7 → 2 Major Flaws. Source: Ars Magica -
    /// Definitive Edition (Core Rules).md:16553-16561.
    #[test]
    fn warping_owed_thresholds_follow_the_score_curve() {
        let owed = |score| WarpingOwed::from_score(score);
        assert_eq!(owed(0), WarpingOwed::default());
        assert_eq!(
            owed(1),
            WarpingOwed {
                minor_flaws: 1,
                minor_supernatural_virtues: 0,
                major_flaws: 0
            }
        );
        assert_eq!(
            owed(3),
            WarpingOwed {
                minor_flaws: 2,
                minor_supernatural_virtues: 0,
                major_flaws: 0
            }
        );
        assert_eq!(
            owed(5),
            WarpingOwed {
                minor_flaws: 2,
                minor_supernatural_virtues: 1,
                major_flaws: 0
            }
        );
        assert_eq!(
            owed(6),
            WarpingOwed {
                minor_flaws: 2,
                minor_supernatural_virtues: 1,
                major_flaws: 1
            }
        );
        assert_eq!(
            owed(7),
            WarpingOwed {
                minor_flaws: 2,
                minor_supernatural_virtues: 1,
                major_flaws: 2
            }
        );
    }

    /// A dedicated ruleset carrying both a non-magus (`companion`) and a magus
    /// (`is_magus`) profile plus an advancement curve, so the magus-exemption can
    /// be checked at the same Warping Score. It ships no Arts, so the engine's
    /// magus-required-Hermetic-role integrity gate is skipped.
    fn magus_owed_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative",
            "magnitude": "free", "category": "special", "entity_kinds": ["character"] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["special", "personality"], "creation_phases": [] },
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["special", "personality"], "is_magus": true,
            "gift_categories": ["hermetic"], "creation_phases": [] }
        ]"#;
        // Includes the engine-required Hermetic abilities: the magus type profile
        // makes `validate_engine_required_roles` demand them of a ruleset that
        // ships an abilities catalogue.
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }
          ],
          "abilities": [
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" }
          ]
        }"#;
        let arts = r#"{ "advancement": [{ "score": 1, "total_xp": 1 }], "arts": [] }"#;
        let characteristics = r#"{
          "start_points": 7, "base_max": 3, "base_min": -3,
          "effective_max": 5, "effective_min": -5,
          "costs": [{ "score": 0, "cost": 0 }]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            characteristics: Some(characteristics),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A non-magus with a Warping Score of 2 (15 stored points → curve score 2)
    /// owes one Minor Flaw; a magus at the SAME high score owes nothing — Warping
    /// gives magi Wizard's Twilight instead (Ars Magica - Definitive Edition (Core Rules).md:16551).
    #[test]
    fn magus_is_exempt_from_owed_warping_vf() {
        let rs = magus_owed_ruleset();
        let mut mundane = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        mundane.warping_points = 15;
        assert_eq!(
            warping_owed(&mundane, &rs),
            WarpingOwed {
                minor_flaws: 1,
                minor_supernatural_virtues: 0,
                major_flaws: 0
            }
        );

        let mut magus = mundane.clone();
        magus.type_id = Id::new("magus");
        assert_eq!(warping_owed(&magus, &rs), WarpingOwed::default());
    }

    /// The recursion guard: choosing `warped_by_magic` (a `WarpingGrant` +5 item)
    /// as an owed-FILL must NOT amplify the Warping Score or the owed count, and
    /// the fill is dropped from the folded grants (so it never re-feeds the score).
    #[test]
    fn warping_grant_fill_does_not_amplify_score_or_owed() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.warping_points = 5; // Warping Score 1 → owes exactly one Minor Flaw.
        let baseline_score = warping_score(&e, &rs);
        let baseline_owed = warping_owed(&e, &rs);
        assert_eq!(baseline_owed.minor_flaws, 1);

        // Fill the owed Minor Flaw slot with the WarpingGrant item.
        e.warping_choices.insert(
            format!("{WARPING_MINOR_FLAW_KEY}0"),
            sel("flaw.warped_by_magic"),
        );

        // Owed count and displayed score are unchanged (no +5 feedback), and the
        // ineligible pick is filtered out of the folded grants.
        assert_eq!(warping_owed(&e, &rs), baseline_owed);
        assert_eq!(warping_score(&e, &rs), baseline_score);
        assert!(
            !warping_granted_selections(&e, &rs).contains(&sel("flaw.warped_by_magic")),
            "a WarpingGrant fill must be dropped from the folded grants"
        );
    }

    /// A legal owed fill folds into `entity_grants` as a real (off-budget)
    /// selection, so prereqs/effects see it — but it never touches the V/F budget.
    #[test]
    fn owed_warping_fill_folds_off_budget() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.warping_points = 5; // owes one Minor Flaw
        e.warping_choices.insert(
            format!("{WARPING_MINOR_FLAW_KEY}0"),
            sel("flaw.weak_characteristics"),
        );
        assert!(
            entity_grants(&e, &rs).contains(&sel("flaw.weak_characteristics")),
            "a chosen owed fill should fold into entity_grants"
        );
    }

    /// Decrepitude XP is the sum of aging points across every Characteristic,
    /// inverted through the (Ability) advancement curve: 17 points → Decrepitude 2
    /// (15 ≤ 17 < 30). Ars Magica - Definitive Edition (Core Rules).md:16617.
    #[test]
    fn decrepitude_score_sums_aging_points_and_inverts() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        assert_eq!(decrepitude_score(&e, &rs), 0);
        e.aging_points.insert(Characteristic::Str, 10);
        e.aging_points.insert(Characteristic::Qik, 7);
        assert_eq!(decrepitude_points_total(&e), 17);
        assert_eq!(decrepitude_score(&e, &rs), 2);
    }

    /// Aging reductions LOWER the effective Characteristic that derived/play stats
    /// use, floored at the rules minimum, but the bought score creation-legality
    /// reads is untouched.
    #[test]
    fn aging_points_derive_the_drop_but_not_creation() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.characteristics.insert(Characteristic::Str, 3);
        // Minimal points that force two drops on a +3 score: (|3|+1)+(|2|+1) = 7.
        e.aging_points.insert(Characteristic::Str, 7);
        assert_eq!(aging_drops(&e, &rs, Characteristic::Str), 2);
        // Derived (aged-down) value is bought − derived drops.
        assert_eq!(
            effective_characteristic_after_aging(&e, &rs, Characteristic::Str),
            1
        );
        // Creation-legality still reads the un-aged bought score.
        assert_eq!(
            e.characteristics.get(&Characteristic::Str).copied(),
            Some(3)
        );
        // The floor clamps at the rules effective minimum (−5), never below.
        e.aging_points.insert(Characteristic::Str, 200);
        assert_eq!(
            effective_characteristic_after_aging(&e, &rs, Characteristic::Str),
            -5
        );
    }

    #[test]
    fn aging_drops_match_the_worked_examples() {
        // Ars Magica - Definitive Edition (Core Rules).md:16613: a Communication of +2 drops to +1 in the year it
        // gains its THIRD aging point; a Stamina of −3 drops to −4 on its FOURTH.
        let rs = xp_ruleset();
        let mut com = xp_entity(vec![]);
        com.characteristics.insert(Characteristic::Com, 2);
        com.aging_points.insert(Characteristic::Com, 2); // ≤ |2|, no drop yet
        assert_eq!(aging_drops(&com, &rs, Characteristic::Com), 0);
        assert_eq!(
            effective_characteristic_after_aging(&com, &rs, Characteristic::Com),
            2
        );
        com.aging_points.insert(Characteristic::Com, 3); // the third point drops it
        assert_eq!(aging_drops(&com, &rs, Characteristic::Com), 1);
        assert_eq!(
            effective_characteristic_after_aging(&com, &rs, Characteristic::Com),
            1
        );

        let mut sta = xp_entity(vec![]);
        sta.characteristics.insert(Characteristic::Sta, -3);
        sta.aging_points.insert(Characteristic::Sta, 3); // ≤ |−3|, no drop yet
        assert_eq!(aging_drops(&sta, &rs, Characteristic::Sta), 0);
        sta.aging_points.insert(Characteristic::Sta, 4); // the fourth point drops it
        assert_eq!(aging_drops(&sta, &rs, Characteristic::Sta), 1);
        assert_eq!(
            effective_characteristic_after_aging(&sta, &rs, Characteristic::Sta),
            -4
        );
    }

    #[test]
    fn aging_drop_applies_to_bought_score_then_free_delta_stacks_on_top() {
        // Decision: the aging drop lowers the *bought* score (its threshold uses
        // the bought score per Ars Magica - Definitive Edition (Core Rules).md:16579/:16613); the free
        // CharacteristicScoreDelta bonus (Giant Blood +1 Str) is then added on
        // top, so an aged Giant-Blood Strength can still reach +6.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.giant_blood")]);
        e.characteristics.insert(Characteristic::Str, 5); // bought 5, +1 delta = 6
        assert_eq!(
            effective_characteristic_after_aging(&e, &rs, Characteristic::Str),
            6
        );
        // One drop: bought 5 → 4, plus the +1 delta = 5.
        e.aging_points.insert(Characteristic::Str, 6); // |5| = 5, sixth point drops
        assert_eq!(aging_drops(&e, &rs, Characteristic::Str), 1);
        assert_eq!(
            effective_characteristic_after_aging(&e, &rs, Characteristic::Str),
            5
        );
    }

    #[test]
    fn effective_summary_maps_report_aged_value_and_drops() {
        // Ars Magica - Definitive Edition (Core Rules).md:16613 worked example: Communication +2 with 3 aging points
        // drops once → effective +1. The summary maps must surface both the aged
        // effective value and the drop count, and omit unchanged Characteristics.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![]);
        e.characteristics.insert(Characteristic::Com, 2);
        e.aging_points.insert(Characteristic::Com, 3);

        let effective = effective_characteristics(&e, &rs);
        assert_eq!(effective.get(&Characteristic::Com).copied(), Some(1));
        // A Characteristic whose effective value equals its bought score is omitted.
        assert!(!effective.contains_key(&Characteristic::Str));

        let drops = characteristic_aging_drops(&e, &rs);
        assert_eq!(drops.get(&Characteristic::Com).copied(), Some(1));
        // Zero-drop Characteristics are omitted from the drop map.
        assert!(!drops.contains_key(&Characteristic::Str));
    }

    #[test]
    fn effective_summary_map_includes_free_delta_without_aging() {
        // A free CharacteristicScoreDelta (Giant Blood +1 Str) makes the effective
        // value differ from the bought score even with no aging, so it appears in
        // the effective map but NOT in the aging-drop map.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.giant_blood")]);
        e.characteristics.insert(Characteristic::Str, 3);
        let effective = effective_characteristics(&e, &rs);
        assert_eq!(effective.get(&Characteristic::Str).copied(), Some(4));
        assert!(characteristic_aging_drops(&e, &rs).is_empty());
    }

    #[test]
    fn size_delta_sums_from_virtues_and_flaws() {
        // Size is a derived stat (base 0) modified by SizeDelta effects
        // (Giant Blood +2, Dwarf -2). Ars Magica - Definitive Edition (Core Rules).md:3975-3978, :5996-5998.
        let rs = xp_ruleset();
        assert_eq!(size(&xp_entity(vec![]), &rs), 0);
        assert_eq!(size(&xp_entity(vec![sel("virtue.giant_blood")]), &rs), 2);
        assert_eq!(size(&xp_entity(vec![sel("flaw.dwarf")]), &rs), -2);
    }

    #[test]
    fn giant_blood_grants_free_characteristic_bonus_reaching_six() {
        // Giant Blood adds a free +1 to Str and Sta that may raise the effective
        // score as high as +6 (Ars Magica - Definitive Edition (Core Rules).md:3975-3978). The bought score is untouched.
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.giant_blood")]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, 5)]);
        assert_eq!(characteristic_score_bonus(&e, &rs, Characteristic::Str), 1);
        assert_eq!(characteristic_score_bonus(&e, &rs, Characteristic::Qik), 0);
        assert_eq!(
            effective_characteristic_score(&e, &rs, Characteristic::Str),
            6
        );
    }

    #[test]
    fn characteristic_bonuses_lists_each_nonzero_free_delta_in_canonical_order() {
        // Giant Blood grants a free +1 to Str and +1 to Sta (Ars Magica - Definitive Edition (Core Rules).md:3975-3978). The
        // accessor surfaces exactly those two nonzero bonuses in canonical
        // Characteristic order, omitting the untouched ones.
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.giant_blood")]);
        assert_eq!(
            characteristic_bonuses(&e, &rs),
            vec![
                CharacteristicBonus {
                    characteristic: Characteristic::Str,
                    bonus: 1,
                },
                CharacteristicBonus {
                    characteristic: Characteristic::Sta,
                    bonus: 1,
                },
            ]
        );
        // With no bonus-granting Virtue, the list is empty.
        assert!(characteristic_bonuses(&xp_entity(vec![]), &rs).is_empty());
    }

    #[test]
    fn weak_characteristics_grants_negative_points_and_nets_with_improved() {
        // Weak Characteristics removes 3 budget points (Ars Magica - Definitive Edition (Core Rules).md:7056-7058); the grant
        // is signed and nets against Improved Characteristics (+3).
        let rs = xp_ruleset();
        let weak = xp_entity(vec![sel("flaw.weak_characteristics")]);
        assert_eq!(characteristic_points_granted(&weak, &rs), -3);
        let netted = xp_entity(vec![
            sel("virtue.improved_characteristics"),
            sel("flaw.weak_characteristics"),
        ]);
        assert_eq!(characteristic_points_granted(&netted, &rs), 0);
    }

    #[test]
    fn ability_score_grant_is_a_free_floor() {
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.second_sight")]);
        // No bought row, yet effective score is the granted 1, costing no XP.
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
            1
        );
        assert_eq!(xp_allocation(&e, &rs).total_demand, 0);
    }

    #[test]
    fn ability_score_floors_lists_each_granted_ability_once_at_its_max() {
        let rs = xp_ruleset();
        let e = xp_entity(vec![sel("virtue.second_sight")]);
        assert_eq!(
            ability_score_floors(&e, &rs),
            vec![AbilityFloor {
                ability: Id::new("ability.second_sight"),
                floor: 1,
            }]
        );
        // No grants → empty.
        assert_eq!(ability_score_floors(&xp_entity(vec![]), &rs), vec![]);
    }

    #[test]
    fn ability_score_grant_floor_does_not_lower_a_higher_bought_score() {
        let rs = xp_ruleset();
        let mut e = xp_entity(vec![sel("virtue.second_sight")]);
        e.ability_scores = vec![plain("ability.second_sight", 3)];
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
            3
        );
    }

    // ---- Phase 4 (step 8): House-granted Virtues feed the effect scanners ----

    use crate::ruleset::RulesetSources;

    /// A ruleset with grant-bearing Houses: Bjornaer grants the (Major, Hermetic)
    /// Heartbeast, whose `AbilityScoreGrant` seeds the Heartbeast Ability at 1;
    /// Flambeau offers a Puissant Art choice (Perdo / Ignem, +3).
    fn house_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
            "category": "special", "entity_kinds": ["character"] },
          { "id": "virtue.heartbeast", "kind": "virtue", "classification": "narrative", "magnitude": "major",
            "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.heartbeast", "amount": 1 }] },
          { "id": "virtue.puissant_art", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
            "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let abilities = r#"{ "abilities": [
          { "id": "ability.heartbeast", "category": "supernatural", "requires_training": true }
        ] }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.perdo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let houses = r#"{ "houses": [
          { "id": "house.bjornaer", "lineage_type": "mystery_cult",
            "grants": [ { "kind": "fixed", "item": "virtue.heartbeast" } ] },
          { "id": "house.flambeau", "lineage_type": "societas",
            "grants": [ { "kind": "choice", "choice_key": "flambeau_puissant", "options": [
              { "ref": "virtue.puissant_art", "params": { "art": "art.perdo" } },
              { "ref": "virtue.puissant_art", "params": { "art": "art.ignem" } }
            ] } ] }
        ] }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: "[]",
            abilities: Some(abilities),
            arts: Some(arts),
            houses: Some(houses),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    fn magus_in_house(house: &str) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        e.house = Some(Id::new(house));
        e
    }

    #[test]
    fn selections_for_effects_borrows_without_grants_and_combines_with_them() {
        let rs = house_ruleset();
        // No House → the bought selections, borrowed unchanged (no clone).
        let mut e = magus_in_house("house.bjornaer");
        e.house = None;
        e.selections = vec![Selection::new(Id::new("virtue.the_gift"))];
        assert_eq!(selections_for_effects(&e, &rs).len(), 1);
        // Bjornaer → the bought row plus the granted Heartbeast row, appended.
        let g = magus_in_house("house.bjornaer");
        let combined = selections_for_effects(&g, &rs);
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].item_ref, Id::new("virtue.heartbeast"));
    }

    #[test]
    fn house_grant_seeds_a_mystery_ability_floor() {
        let rs = house_ruleset();
        let e = magus_in_house("house.bjornaer");
        // Bjornaer grants Heartbeast, whose AbilityScoreGrant floors the
        // Heartbeast Ability at 1 — no bought row, no XP charged.
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.heartbeast"), None),
            1
        );
        assert_eq!(xp_allocation(&e, &rs).total_demand, 0);
    }

    #[test]
    fn granted_puissant_art_adds_to_effective_art_score() {
        let rs = house_ruleset();
        let mut e = magus_in_house("house.flambeau");
        e.house_choices.insert(
            "flambeau_puissant".into(),
            Selection::with_params(
                Id::new("virtue.puissant_art"),
                BTreeMap::from([("art".into(), Id::new("art.ignem"))]),
            ),
        );
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 2,
        }];
        // The House-granted Puissant Ignem (+3) stacks on the bought score.
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 5);
        // A non-targeted Art is untouched by the grant.
        assert_eq!(art_bonus(&e, &rs, &Id::new("art.perdo")), 0);
    }

    // --- Elemental Magic (5c): XP-space Art-XP redistribution ---

    /// A ruleset with the four elemental Forms (Aquam, Auram, Ignem, Terram) plus a
    /// non-elemental Form (Corpus) and a Technique (Creo), the full triangular Art
    /// advancement curve out to score 10, `virtue.elemental_magic` carrying the
    /// `ElementalMagic` marker over the four Forms, and Puissant Art.
    fn elemental_ruleset() -> Ruleset {
        let items = r#"[
          {
            "id": "virtue.elemental_magic",
            "kind": "virtue",
            "classification": "creation_effect",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "effects": [{
              "type": "elemental_magic",
              "forms": ["art.aquam", "art.auram", "art.ignem", "art.terram"]
            }]
          },
          {
            "id": "virtue.puissant_art",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }]
          },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          {
            "id": "companion",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "hermetic"],
            "forbidden_categories": [],
            "required_traits": [],
            "forbidden_traits": [],
            "gift_policy": "allowed",
            "gift_categories": [],
            "creation_phases": ["concept"]
          }
        ]"#;
        let abilities = r#"{ "abilities": [
          { "id": "ability.awareness", "category": "general" }
        ] }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }, { "score": 6, "total_xp": 21 },
            { "score": 7, "total_xp": 28 }, { "score": 8, "total_xp": 36 },
            { "score": 9, "total_xp": 45 }, { "score": 10, "total_xp": 55 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.aquam", "art_type": "form" },
            { "id": "art.auram", "art_type": "form" },
            { "id": "art.corpus", "art_type": "form" },
            { "id": "art.ignem", "art_type": "form" },
            { "id": "art.terram", "art_type": "form" }
          ]
        }"#;
        let characteristics = r#"{
          "start_points": 7,
          "base_max": 3, "base_min": -3,
          "effective_max": 5, "effective_min": -5,
          "costs": [
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 },
            { "score": 1, "cost": 1 }, { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 },
            { "score": -3, "cost": -6 }
          ]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            characteristics: Some(characteristics),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn art_row(art: &str, score: u8) -> ArtScore {
        ArtScore {
            art: Id::new(art),
            score,
        }
    }

    /// Three elemental Forms bought at score 6 (21 table-XP each) and one at score 4
    /// (10 table-XP): the boosted effective scores match the ceil-rounded
    /// redistribution hand-computed from the worked example (Ars Magica - Definitive Edition (Core Rules).md:3731-3737).
    ///
    /// bonus_xp(F) = Σ_{G≠F} ceil(xp(G)/2):
    ///   Aquam/Auram/Ignem (own 21): 11 + 11 + 5 = 27 → 48 XP → score 9.
    ///   Terram (own 10):            11 + 11 + 11 = 33 → 43 XP → score 8.
    #[test]
    fn elemental_magic_redistributes_art_xp_rounded_up() {
        let rs = elemental_ruleset();
        let mut e = entity(vec![Selection::new(Id::new("virtue.elemental_magic"))]);
        e.art_scores = vec![
            art_row("art.aquam", 6),
            art_row("art.auram", 6),
            art_row("art.ignem", 6),
            art_row("art.terram", 4),
        ];
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.aquam")), 9);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.auram")), 9);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 9);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 8);
        // The boost surfaces through art_bonuses (delta over the bought score), so
        // the UI shows it like a Puissant Art bonus.
        let bonuses = art_bonuses(&e, &rs);
        let bonus = |art: &str| {
            bonuses
                .iter()
                .find(|b| b.art == Id::new(art))
                .map(|b| b.bonus)
        };
        assert_eq!(bonus("art.aquam"), Some(3));
        assert_eq!(bonus("art.terram"), Some(4));
    }

    /// Regression (Issue 13): an elemental Form with 0 bought points still earns a
    /// redistribution boost from the other Forms, and it must surface through
    /// `art_bonuses` even though it has no `art_scores` row.
    #[test]
    fn art_bonuses_include_elemental_boost_at_bought_zero() {
        let rs = elemental_ruleset();
        let mut e = entity(vec![Selection::new(Id::new("virtue.elemental_magic"))]);
        // Terram is not bought (no row); the other three Forms feed its boost.
        e.art_scores = vec![
            art_row("art.aquam", 6),
            art_row("art.auram", 6),
            art_row("art.ignem", 6),
        ];
        // bonus_xp(Terram) = 3 * ceil(21/2) = 33 → score 7; delta over bought 0 = 7.
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 7);
        let bonuses = art_bonuses(&e, &rs);
        assert_eq!(
            bonuses
                .iter()
                .find(|b| b.art == Id::new("art.terram"))
                .map(|b| b.bonus),
            Some(7)
        );
    }

    /// A magus without the marker gets no redistribution: effective == bought.
    #[test]
    fn non_elemental_magus_unaffected_by_redistribution() {
        let rs = elemental_ruleset();
        let mut e = entity(vec![]);
        e.art_scores = vec![
            art_row("art.aquam", 6),
            art_row("art.ignem", 6),
            art_row("art.terram", 4),
        ];
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.aquam")), 6);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 6);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 4);
        assert!(art_bonuses(&e, &rs).is_empty());
    }

    /// Redistribution touches ONLY the four elemental Forms — never a Technique or a
    /// non-elemental Form, even when they carry a score that would earn a big bonus.
    #[test]
    fn elemental_magic_scoped_to_the_four_forms() {
        let rs = elemental_ruleset();
        let mut e = entity(vec![Selection::new(Id::new("virtue.elemental_magic"))]);
        e.art_scores = vec![
            art_row("art.aquam", 6),
            art_row("art.auram", 6),
            art_row("art.ignem", 6),
            art_row("art.terram", 6),
            art_row("art.corpus", 6),
            art_row("art.creo", 6),
        ];
        // Corpus (non-elemental Form) and Creo (Technique) stay at their bought score.
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.corpus")), 6);
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.creo")), 6);
    }

    /// Puissant Art (a flat XP-free bonus) stacks on top of the XP-space boost.
    #[test]
    fn elemental_boost_and_puissant_stack() {
        let rs = elemental_ruleset();
        let mut e = entity(vec![
            Selection::new(Id::new("virtue.elemental_magic")),
            puissant_art("art.ignem"),
        ]);
        e.art_scores = vec![
            art_row("art.aquam", 6),
            art_row("art.auram", 6),
            art_row("art.ignem", 6),
            art_row("art.terram", 4),
        ];
        // Ignem: boosted to 9 by redistribution, then +3 Puissant = 12.
        assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 12);
    }

    /// The redistribution is a free derived bonus — it adds no XP-pool demand.
    #[test]
    fn elemental_magic_does_not_perturb_xp_pool_spend() {
        let rs = elemental_ruleset();
        let scores = vec![
            art_row("art.aquam", 6),
            art_row("art.auram", 6),
            art_row("art.ignem", 6),
            art_row("art.terram", 4),
        ];
        let mut with = entity(vec![Selection::new(Id::new("virtue.elemental_magic"))]);
        with.art_scores = scores.clone();
        let mut without = entity(vec![]);
        without.art_scores = scores;
        // Same bought scores fund the same demand with or without the marker
        // (21+21+21+10 = 73), so the free boost never inflates the pool cost.
        assert_eq!(xp_allocation(&with, &rs).total_demand, 73);
        assert_eq!(
            xp_allocation(&with, &rs).total_demand,
            xp_allocation(&without, &rs).total_demand
        );
    }

    /// The per-Technique/Form spell-level caps surfaced to the UI equal
    /// Te + Fo + Int + Magic Theory + 3 (Ars Magica - Definitive Edition (Core Rules).md:2465), one entry per Te×Fo combo.
    #[test]
    fn spell_level_caps_expose_te_fo_int_mt_plus_three() {
        let rs = ruleset();
        let mut e = entity(vec![]);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 2,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 3,
            },
        ];
        e.characteristics.insert(Characteristic::Int, 1);
        let caps = spell_level_caps(&e, &rs);
        // The fixture has one Technique (Creo) × one Form (Ignem) → one combo.
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].technique, Id::new("art.creo"));
        assert_eq!(caps[0].form, Id::new("art.ignem"));
        // 2 (Creo) + 3 (Ignem) + 1 (Int) + 0 (no Magic Theory) + 3 = 9.
        assert_eq!(caps[0].cap, 9);
    }

    /// Issue 11: with no per-character override the base budget is the type
    /// profile's `spell_levels` (120 in the fixture), and it feeds the budget.
    #[test]
    fn spell_levels_base_without_override_uses_profile() {
        let rs = ruleset();
        let profile = rs.profile(&Id::new("companion"));
        let e = entity(vec![]);
        assert_eq!(e.spell_levels_override, None);
        assert_eq!(spell_levels_base(&e, profile), 120);
        let base = spell_levels_base(&e, profile);
        assert_eq!(spell_levels_budget(base, &e, &rs), 120);
    }

    /// Issue 11: an explicit per-character override REPLACES the profile base;
    /// Skilled Parens's +30 [`Effect::SpellLevels`] bonus still adds on top.
    #[test]
    fn spell_levels_override_replaces_profile_base() {
        let rs = ruleset();
        let profile = rs.profile(&Id::new("companion"));
        let mut e = entity(vec![Selection::new(Id::new("virtue.skilled_parens"))]);
        e.spell_levels_override = Some(80);
        // base = override 80 (not the profile's 120); + 30 Skilled Parens = 110.
        assert_eq!(spell_levels_base(&e, profile), 80);
        let base = spell_levels_base(&e, profile);
        assert_eq!(spell_levels_budget(base, &e, &rs), 110);
    }

    /// The V/F contribution is surfaced on its own (not only folded into the
    /// budget), so the spell-levels bar can show it beside the base and the
    /// post-Gauntlet levels the way the XP bar lists its extra pools beside the
    /// general one.
    #[test]
    fn spell_levels_bonus_is_reported_separately_from_the_base() {
        let rs = ruleset();
        // No spell-levels V/F → no bonus to report.
        assert_eq!(spell_levels_bonus(&entity(vec![]), &rs), 0);
        // Skilled Parens contributes its +30 as a standalone signed figure.
        let e = entity(vec![Selection::new(Id::new("virtue.skilled_parens"))]);
        assert_eq!(spell_levels_bonus(&e, &rs), 30);
    }

    /// The general pool's V/F contribution is surfaced on its own for the same
    /// reason the spell-levels one is: the pool the solve funds from is `typed +
    /// bonus`, and a bar that shows only the total cannot say why the two differ.
    /// Skilled Parens: "You gain an additional 60 experience points … during
    /// apprenticeship" (Ars Magica - Definitive Edition (Core Rules).md:4966).
    #[test]
    fn the_general_xp_bonus_is_reported_beside_the_pool_it_raises() {
        let rs = ruleset();
        let mut plain = entity(vec![]);
        plain.xp_pool = 240;
        let allocation = xp_allocation(&plain, &rs);
        assert_eq!(allocation.general_bonus, 0);
        assert_eq!(allocation.general_pool, 240);

        let mut skilled = plain.clone();
        skilled.selections = vec![Selection::new(Id::new("virtue.skilled_parens"))];
        let allocation = xp_allocation(&skilled, &rs);
        assert_eq!(allocation.general_bonus, 60);
        // The identity the bar's split closes against: pool = typed + bonus.
        assert_eq!(allocation.general_pool, 300);
    }

    // --- life-stage pools (M6b2) ---------------------------------------------

    /// A ruleset with life stages, a parameterized Living Language, and two of the
    /// childhood spread's Abilities.
    fn life_stage_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] },
          { "id": "virtue.affinity_with_ability", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "affinity_ability_cost", "param": "ability",
                          "counts_as_num": 3, "counts_as_den": 2 }] },
          { "id": "virtue.puissant_ability", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }] },
          { "id": "virtue.skilled_parens", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "general_xp", "amount": 60 }] },
          { "id": "flaw.covenant_upbringing", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "ability_authorization",
                          "abilities": ["ability.dead_language"] }] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "personality"], "creation_phases": [] },
          { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "personality", "hermetic"],
            "is_magus": true, "spell_levels": 120, "creation_phases": [] }
        ]"#;
        // Ability table: 5/15/30/50/75 — the shipped Core Rules figures. The five
        // Hermetic roles are present because the magus profile above obliges any
        // ruleset shipping an abilities catalogue to carry them.
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "categories_requiring_virtue": ["academic", "arcane", "martial"],
          "abilities": [
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.concentration", "category": "general" },
            { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
            { "id": "ability.living_language", "category": "general", "parameter": "language" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" },
            { "id": "ability.swim", "category": "general" }
          ]
        }"#;
        // Art table: the shipped Core Rules figures up to 20 (`:2408-2427`), so a
        // post-Gauntlet magus can buy an Art far beyond what apprenticeship's 240
        // could ever fund.
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }, { "score": 6, "total_xp": 21 },
            { "score": 7, "total_xp": 28 }, { "score": 8, "total_xp": 36 },
            { "score": 9, "total_xp": 45 }, { "score": 10, "total_xp": 55 },
            { "score": 11, "total_xp": 66 }, { "score": 12, "total_xp": 78 },
            { "score": 13, "total_xp": 91 }, { "score": 14, "total_xp": 105 },
            { "score": 15, "total_xp": 120 }, { "score": 16, "total_xp": 136 },
            { "score": 17, "total_xp": 153 }, { "score": 18, "total_xp": 171 },
            { "score": 19, "total_xp": 190 }, { "score": 20, "total_xp": 210 }
          ],
          "arts": [
            { "id": "art.corpus", "art_type": "form" },
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let life_stages = r#"{
          "apprenticeship": {
            "years": 15,
            "xp": 240,
            "minimum_abilities": [],
            "recommended_abilities": [],
            "recommended_xp": 0
          },
          "childhood": {
            "years": 5,
            "native_language_ability": "ability.living_language",
            "native_language_xp": 75,
            "spread_xp": 45,
            "spread_abilities": ["ability.living_language", "ability.swim"]
          },
          "later_life": { "xp_per_year": 15 },
          "post_apprenticeship": {
            "lab_season_cost": 10,
            "max_charged_lab_seasons_per_year": 3,
            "points_per_year": 30
          }
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            life_stages: Some(life_stages),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A 25-year-old companion whose childhood bought Native Language (German) 5.
    fn planned_companion() -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = Some(25);
        entity.life_stages = Some(crate::life_stage::LifeStagePlan {
            native_language: Some("German".into()),
            ..crate::life_stage::LifeStagePlan::default()
        });
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.living_language"),
            parameter: Some("German".into()),
            score: 5,
            specialty: None,
        }];
        entity
    }

    /// The same, as a magus: 25 years old, so taken as an apprentice at 10 and
    /// standing at its Gauntlet.
    fn planned_magus() -> Entity {
        let mut entity = planned_companion();
        entity.type_id = Id::new("magus");
        entity
    }

    /// A guided magus's **general** pool is its apprenticeship experience, not its
    /// later life: "These experience points can be spent on Arts or Abilities"
    /// (Ars Magica - Definitive Edition (Core Rules).md:2435), and the general pool is the only one that may fund an
    /// Art. Later life buys "any Abilities" (`:2214`) and becomes a restricted pool
    /// of its own.
    #[test]
    fn the_general_pool_of_a_guided_magus_is_its_apprenticeship() {
        let rs = life_stage_ruleset();
        assert_eq!(xp_allocation(&planned_magus(), &rs).general_pool, 240);

        // Skilled Parens grants "an additional 60 experience points … during
        // apprenticeship" (`:4966`), which lands on exactly this pool. So the block's
        // base (240) and the pool the solve funds from (300) are different numbers —
        // which is why the base is not stored as a pool anywhere.
        let mut magus = planned_magus();
        magus.selections = vec![Selection::new(Id::new("virtue.skilled_parens"))];
        assert_eq!(xp_allocation(&magus, &rs).general_pool, 300);

        // A companion's general pool is still its later life.
        assert_eq!(xp_allocation(&planned_companion(), &rs).general_pool, 300);
    }

    /// The native-language block funds the native instance and nothing else, so a
    /// Native Language 5 (75 xp) is paid entirely by it — leaving the general pool
    /// (later life) untouched.
    #[test]
    fn the_native_language_block_pays_for_the_native_language() {
        let rs = life_stage_ruleset();
        let allocation = xp_allocation(&planned_companion(), &rs);
        assert_eq!(allocation.total_demand, 75);
        assert_eq!(allocation.max_flow, 75, "the spend is fully funded");
        assert_eq!(allocation.general_used, 0, "later life pays none of it");
        assert_eq!(allocation.general_pool, 300, "20 years at 15 a year");
    }

    /// A childhood-list Ability is paid by the 45-point spread, not by later life.
    #[test]
    fn the_spread_block_pays_for_a_childhood_ability() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.ability_scores.push(AbilityScore {
            ability: Id::new("ability.swim"),
            parameter: None,
            score: 2,
            specialty: None,
        });
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.total_demand, 90, "75 + 15");
        assert_eq!(allocation.max_flow, 90);
        assert_eq!(allocation.general_used, 0, "childhood covers both");
    }

    /// An Ability outside the childhood list can only come from later life.
    #[test]
    fn an_ability_off_the_childhood_list_falls_to_later_life() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.ability_scores.push(AbilityScore {
            ability: Id::new("ability.concentration"),
            parameter: None,
            score: 2,
            specialty: None,
        });
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.max_flow, 90, "fully funded");
        assert_eq!(allocation.general_used, 15, "later life pays the 15");
    }

    /// The native-language block is restricted to the NATIVE instance: a second
    /// Living Language is a childhood-spread purchase (":2378" allows a Living
    /// Language "other than the character's native language"), so it may draw the
    /// 45 but never the 75.
    #[test]
    fn a_second_living_language_draws_the_spread_not_the_native_block() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.ability_scores.push(AbilityScore {
            ability: Id::new("ability.living_language"),
            parameter: Some("French".into()),
            score: 3, // 30 xp: more than the 45 spread can spare alongside nothing else
            specialty: None,
        });
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.total_demand, 105, "75 + 30");
        assert_eq!(allocation.max_flow, 105);
        // The spread (45) covers the French 30; the native block cannot, and later
        // life is not needed.
        assert_eq!(allocation.general_used, 0);

        // Push the second language past what the spread can fund and later life
        // picks up the rest — proof the 75 stays reserved for the native instance.
        entity.ability_scores[1].score = 5; // 75 xp
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.total_demand, 150);
        assert_eq!(allocation.max_flow, 150);
        assert_eq!(allocation.general_used, 30, "75 − 45 comes from later life");
    }

    /// Affinity's discount applies to a childhood-funded Ability exactly as it does
    /// to any other spend: the pool is where the points come from, not how they are
    /// priced. Swim 2 costs 15, so ⌈15 × 2/3⌉ = 10.
    #[test]
    fn affinity_still_discounts_a_childhood_funded_ability() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.selections = vec![Selection::with_params(
            Id::new("virtue.affinity_with_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.swim"))]),
        )];
        entity.ability_scores.push(AbilityScore {
            ability: Id::new("ability.swim"),
            parameter: None,
            score: 2,
            specialty: None,
        });
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.total_demand, 85, "75 + ceil(15 * 2/3) = 75 + 10");
    }

    /// Puissant adds to *use*, not to the bought score, so it is charged nothing —
    /// the opposite handling from Affinity, and worth pinning beside it.
    #[test]
    fn puissant_costs_no_life_stage_experience() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.selections = vec![Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.swim"))]),
        )];
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(
            allocation.total_demand, 75,
            "only the native language is charged"
        );
    }

    /// Without a plan nothing changes: the general pool is the typed `xp_pool` and
    /// no life-stage pool exists. This is the regression guard for every
    /// directly-entered character.
    #[test]
    fn direct_entry_allocation_is_unchanged() {
        let rs = life_stage_ruleset();
        let mut entity = planned_companion();
        entity.life_stages = None;
        entity.xp_pool = 120;
        let allocation = xp_allocation(&entity, &rs);
        assert_eq!(allocation.general_pool, 120);
        assert_eq!(allocation.general_used, 75);
        assert!(
            allocation.restricted.is_empty(),
            "no life-stage pools: {:?}",
            allocation.restricted
        );
    }

    /// The later-life pool of a guided magus, or `None` when it has none.
    fn later_life_pool(allocation: &XpAllocation) -> Option<&RestrictedXpPool> {
        allocation.restricted.iter().find(|pool| {
            pool.origin
                == XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::LaterLife,
                }
        })
    }

    /// A magus's pre-apprenticeship experience may not buy an Arcane, Academic or
    /// Martial Ability: "Note that magi can only spend experience points on Arcane,
    /// Academic and Martial Abilities **before** apprenticeship if they have a Virtue
    /// which allows them to do so." (Ars Magica - Definitive Edition (Core Rules).md:2435.) The Darius example reasons
    /// the same way about a pre-apprenticeship purchase — "It's a **general** Ability,
    /// so he can" (`:2402`).
    ///
    /// So later life is a pool of its own for a magus, and a gated Ability falls to
    /// apprenticeship's general pool instead.
    #[test]
    fn pre_apprenticeship_experience_cannot_buy_a_gated_ability() {
        let rs = life_stage_ruleset();
        let mut magus = planned_magus();
        // Concentration 3 — a General Ability, 30 experience points.
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.concentration"),
            parameter: None,
            score: 3,
            specialty: None,
        });
        let allocation = xp_allocation(&magus, &rs);
        let pool = later_life_pool(&allocation).expect("a guided magus has a later-life pool");
        assert_eq!(pool.amount, 75, "five years at 15 a year");
        assert_eq!(pool.used, 30, "later life buys the General Ability");
        assert_eq!(allocation.general_used, 0, "apprenticeship pays none of it");

        // Artes Liberales 3 — Academic, so the same 30 points cannot come from
        // before apprenticeship, and the apprenticeship pool takes it instead.
        magus.ability_scores.pop();
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            parameter: None,
            score: 3,
            specialty: None,
        });
        let allocation = xp_allocation(&magus, &rs);
        assert_eq!(later_life_pool(&allocation).expect("still there").used, 0);
        assert_eq!(allocation.general_used, 30, "apprenticeship funds it");

        // A companion has no such pool at all: its later life IS the general pool.
        assert!(later_life_pool(&xp_allocation(&planned_companion(), &rs)).is_none());
    }

    /// …unless a Virtue says otherwise: "if they have a Virtue which allows them to do
    /// so" (Ars Magica - Definitive Edition (Core Rules).md:2435). Covenant Upbringing authorizes the dead language
    /// ("You may take Latin at character creation", `:5867`), so those points may come
    /// from before apprenticeship after all.
    ///
    /// A **regression lock**: the pool is built from the same authorizations the
    /// ownership check reads, so this already holds — which is exactly the property
    /// worth pinning, since the two readings may never drift apart.
    #[test]
    fn an_authorizing_virtue_lets_pre_apprenticeship_experience_buy_a_gated_ability() {
        let rs = life_stage_ruleset();
        let mut magus = planned_magus();
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.dead_language"),
            parameter: Some("Latin".into()),
            score: 3,
            specialty: None,
        });

        // Without the Flaw, the 30 points must come from apprenticeship.
        let allocation = xp_allocation(&magus, &rs);
        assert_eq!(later_life_pool(&allocation).expect("a pool").used, 0);
        assert_eq!(allocation.general_used, 30);

        // With it, later life may fund them.
        magus.selections = vec![Selection::new(Id::new("flaw.covenant_upbringing"))];
        let allocation = xp_allocation(&magus, &rs);
        assert_eq!(later_life_pool(&allocation).expect("a pool").used, 30);
        assert_eq!(allocation.general_used, 0);

        // It authorizes that Ability alone: Artes Liberales stays out.
        magus.ability_scores.pop();
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            parameter: None,
            score: 3,
            specialty: None,
        });
        let allocation = xp_allocation(&magus, &rs);
        assert_eq!(later_life_pool(&allocation).expect("a pool").used, 0);
        assert_eq!(allocation.general_used, 30);
    }

    /// A magus gauntleted at 25 and now 60: thirty-five years of "30 points per
    /// year" (Ars Magica - Definitive Edition (Core Rules).md:2471) behind it, none of them spent in the lab.
    fn experienced_magus() -> Entity {
        let mut magus = planned_magus();
        magus.age = Some(60);
        magus.life_stages = Some(crate::life_stage::LifeStagePlan {
            gauntlet_age: Some(25),
            ..magus
                .life_stages
                .clone()
                .expect("a planned magus has a plan")
        });
        magus
    }

    /// The years after the Gauntlet fund the **general** pool: "Divide 30 points per
    /// year between experience points in Arts, experience points in Abilities, and
    /// levels of spells" (Ars Magica - Definitive Edition (Core Rules).md:2216) — Arts included, which no restricted
    /// pool may ever fund.
    ///
    /// Proved through the allocation rather than by reading the budget: two Arts at
    /// 20 cost 420, well past the 240 apprenticeship alone could pay, so the demand
    /// is only fully funded if the post-Gauntlet experience is in the general pool.
    #[test]
    fn post_gauntlet_experience_buys_arts_from_the_general_pool() {
        let rs = life_stage_ruleset();
        let mut magus = experienced_magus();
        magus.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 20,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 20,
            },
        ];
        let allocation = xp_allocation(&magus, &rs);
        assert_eq!(
            allocation.general_pool, 1290,
            "240 apprenticeship + 35 years at 30 a year"
        );
        assert_eq!(
            allocation.total_demand, 495,
            "75 native language + 420 Arts"
        );
        assert_eq!(allocation.max_flow, 495, "fully funded");
        assert_eq!(
            allocation.general_used, 420,
            "the general pool pays the Arts"
        );
    }

    /// The post-Gauntlet experience does not widen later life: that block stays the
    /// restricted, Abilities-only pool of `:2435`'s "before apprenticeship" clause,
    /// so it funds neither an Art nor a gated Academic Ability however many years
    /// the magus has lived since.
    #[test]
    fn post_gauntlet_years_leave_later_life_restricted() {
        let rs = life_stage_ruleset();
        let mut magus = experienced_magus();
        magus.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }];
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            parameter: None,
            score: 3,
            specialty: None,
        });
        let allocation = xp_allocation(&magus, &rs);
        let pool = later_life_pool(&allocation).expect("an experienced magus still has one");
        assert_eq!(pool.amount, 75, "five years at 15 a year, unchanged");
        assert_eq!(
            pool.used, 0,
            "later life buys neither the Art nor the Academic Ability"
        );
        assert_eq!(allocation.general_used, 45, "15 for the Art + 30 for Latin");
    }

    /// A companion's allocation is untouched by post-Gauntlet fields on its plan:
    /// "**Hermetic Magi Only (Optional):** Years after apprenticeship" (`:2216`), and
    /// a companion serves no apprenticeship, so the numbers may not move a point.
    #[test]
    fn a_companion_allocation_ignores_post_gauntlet_fields() {
        let rs = life_stage_ruleset();
        let plain = planned_companion();
        let mut annotated = planned_companion();
        annotated.life_stages = Some(crate::life_stage::LifeStagePlan {
            gauntlet_age: Some(20),
            post_gauntlet_lab_seasons: 4,
            post_gauntlet_spell_levels: 100,
            ..annotated.life_stages.clone().expect("a plan")
        });
        assert_eq!(xp_allocation(&annotated, &rs), xp_allocation(&plain, &rs));
    }

    /// A magus standing at its Gauntlet has lived no year past it, so its general
    /// pool is the apprenticeship 240 and nothing more — the pre-6b5 number, held
    /// even when the plan carries lab seasons and spell levels a Gauntlet age would
    /// have made meaningful.
    #[test]
    fn a_magus_at_its_gauntlet_has_only_its_apprenticeship() {
        let rs = life_stage_ruleset();
        let mut magus = planned_magus();
        magus.life_stages = Some(crate::life_stage::LifeStagePlan {
            post_gauntlet_lab_seasons: 4,
            post_gauntlet_spell_levels: 100,
            ..magus.life_stages.clone().expect("a plan")
        });
        assert_eq!(xp_allocation(&magus, &rs).general_pool, 240);
    }

    /// The levels of spells a magus took out of its post-Gauntlet points are **added
    /// to** the profile's 120, not a second budget beside it: the 120 of `:2435` are
    /// the type profile's `spell_levels`, while these are the player's chosen slice of
    /// the fungible "30 points per year" (`:2471`).
    #[test]
    fn post_gauntlet_spell_levels_add_to_the_profile_budget() {
        let rs = life_stage_ruleset();
        let profile = rs.profile(&Id::new("magus"));
        let mut magus = experienced_magus();
        magus.life_stages = Some(crate::life_stage::LifeStagePlan {
            post_gauntlet_spell_levels: 300,
            ..magus.life_stages.clone().expect("a plan")
        });
        assert_eq!(life_stage_spell_levels(&magus, &rs), 300);
        let base = spell_levels_base(&magus, profile);
        assert_eq!(base, 120, "the profile's apprenticeship levels, untouched");
        assert_eq!(spell_levels_budget(base, &magus, &rs), 420, "120 + 300");
    }

    /// A magus standing at its Gauntlet has no post-Gauntlet points to slice, so its
    /// budget is the profile's 120 — the pre-6b5 number.
    #[test]
    fn a_magus_at_its_gauntlet_keeps_the_profile_spell_budget() {
        let rs = life_stage_ruleset();
        let profile = rs.profile(&Id::new("magus"));
        let magus = planned_magus();
        assert_eq!(life_stage_spell_levels(&magus, &rs), 0);
        let base = spell_levels_base(&magus, profile);
        assert_eq!(spell_levels_budget(base, &magus, &rs), 120);
    }

    /// `spell_levels_override` replaces the **profile base** only; the post-Gauntlet
    /// levels stay additive on top of whatever base is in force. Deliberate: the
    /// override is the flat flow's escape hatch and is not made exclusive with a plan
    /// the way `xp_pool` is.
    #[test]
    fn an_override_replaces_the_base_not_the_post_gauntlet_levels() {
        let rs = life_stage_ruleset();
        let profile = rs.profile(&Id::new("magus"));
        let mut magus = experienced_magus();
        magus.life_stages = Some(crate::life_stage::LifeStagePlan {
            post_gauntlet_spell_levels: 300,
            ..magus.life_stages.clone().expect("a plan")
        });
        magus.spell_levels_override = Some(80);
        let base = spell_levels_base(&magus, profile);
        assert_eq!(base, 80, "the override replaces the profile's 120");
        assert_eq!(spell_levels_budget(base, &magus, &rs), 380, "80 + 300");
    }

    /// A non-magus earns none: "**Hermetic Magi Only (Optional):** Years after
    /// apprenticeship" (`:2216`), so a companion's plan contributes 0 however its
    /// post-Gauntlet fields are set — and so does a character with no plan at all.
    #[test]
    fn a_non_magus_gets_no_post_gauntlet_spell_levels() {
        let rs = life_stage_ruleset();
        let mut companion = planned_companion();
        companion.life_stages = Some(crate::life_stage::LifeStagePlan {
            gauntlet_age: Some(20),
            post_gauntlet_spell_levels: 100,
            ..companion.life_stages.clone().expect("a plan")
        });
        assert_eq!(life_stage_spell_levels(&companion, &rs), 0);

        let mut direct = planned_magus();
        direct.life_stages = None;
        assert_eq!(life_stage_spell_levels(&direct, &rs), 0);
    }

    /// Later life is a life-stage block like the childhood ones, because for a magus
    /// it is a **restricted** pool: it buys "any Abilities" (Ars Magica - Definitive Edition (Core Rules).md:2214,
    /// `:2392`) and never an Art, which only apprenticeship's experience may. So it
    /// needs a slug of its own, and a Fluent label — a block the UI cannot name would
    /// print its own slug.
    #[test]
    fn life_stage_blocks_include_later_life() {
        assert_eq!(LifeStageBlock::ALL.len(), 3);
        assert!(LifeStageBlock::ALL.contains(&LifeStageBlock::LaterLife));
        assert_eq!(LifeStageBlock::LaterLife.to_string(), "later_life");
    }

    /// Each life-stage pool says where it came from, so the XP bar can label it
    /// through a Fluent key rather than guessing from its ability list.
    #[test]
    fn life_stage_pools_carry_their_origin() {
        let rs = life_stage_ruleset();
        let allocation = xp_allocation(&planned_companion(), &rs);
        let origins: Vec<&XpPoolOrigin> = allocation.restricted.iter().map(|p| &p.origin).collect();
        assert!(
            origins.contains(&&XpPoolOrigin::LifeStage {
                block: LifeStageBlock::ChildhoodNativeLanguage
            }),
            "origins: {origins:?}"
        );
        assert!(
            origins.contains(&&XpPoolOrigin::LifeStage {
                block: LifeStageBlock::ChildhoodSpread
            }),
            "origins: {origins:?}"
        );
    }
}
