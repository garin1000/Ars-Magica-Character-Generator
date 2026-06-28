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

use crate::characteristics::Characteristic;
use crate::ruleset::Ruleset;
use crate::types::{Effect, Entity, Id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A non-zero ability-score bonus targeting one ability *instance*. For a
/// parameterized ability ((Area) Lore) the instance is identified by
/// `(ability, parameter)`; a plain ability has `parameter: None`.
///
/// Serializes for the frontend as `{ "ability": "<id>", "bonus": N }`, with
/// `parameter` added only when present (`None` is omitted, never `null`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityBonus {
    /// The slug id of the boosted ability (e.g. `ability.area_lore`).
    pub ability: Id,
    /// The instance discriminator for a parameterized ability ((Area) Lore →
    /// the area name); `None` for a plain ability, which has a single instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The summed bonus for this instance: all matching ability-bonus effects
    /// (e.g. Puissant Ability +2) added together, so stacking virtues combine.
    pub bonus: i32,
}

/// Sum of all ability-bonus effects (e.g. Puissant Ability) targeting one
/// ability instance. The instance is `(ability, parameter)`: a parameterized
/// ability ((Area) Lore) needs the selection to name the same instance under the
/// ability's own param key, so Puissant "Brandenburg Lore" boosts only that area
/// and not "Berlin Lore". A plain ability matches by id alone. Two virtues
/// boosting the same instance stack.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4814-4816 ("You may
/// only take this Virtue once for a given Ability"; each (Area) Lore is a
/// distinct Ability).
pub fn ability_bonus(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> i32 {
    // The instance-discriminator key for a parameterized ability ((Area) Lore →
    // "area"); `None` for a plain ability (a single instance, matched by id).
    let instance_key = ruleset
        .abilities
        .get(ability)
        .and_then(|a| a.parameter.as_deref());
    let mut bonus = 0;
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored bonus.
            match effect {
                Effect::AbilityBonus { param, amount }
                    if selection.params.get(param) == Some(ability) =>
                {
                    let matches = match instance_key {
                        None => true,
                        // The selection must name this instance; one that omits
                        // the instance key targets no parameterized instance at
                        // all.
                        Some(key) => match selection.params.get(key) {
                            Some(named) => Some(named.as_str()) == parameter,
                            None => false,
                        },
                    };
                    if matches {
                        bonus += i32::from(*amount);
                    }
                }
                // Not an ability bonus for this target; contributes nothing here.
                Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. } => {}
            }
        }
    }
    bonus
}

/// The effective score of the `(ability, parameter)` instance: the highest bought
/// score the entity holds for that exact instance plus its bonus. An instance the
/// entity has not bought counts as 0.
pub fn effective_ability_score(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> i32 {
    let bought = entity
        .ability_scores
        .iter()
        .filter(|a| &a.ability == ability && a.parameter.as_deref() == parameter)
        .map(|a| i32::from(a.score))
        .max()
        .unwrap_or(0);
    bought + ability_bonus(entity, ruleset, ability, parameter)
}

/// A non-zero score bonus targeting one Art. Serializes for the frontend as
/// `{ "art": "<id>", "bonus": N }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtBonus {
    /// The slug id of the boosted Art (e.g. `art.ignem`).
    pub art: Id,
    /// The summed bonus: all matching art-bonus effects (e.g. Puissant Art +3)
    /// added together, so stacking virtues combine.
    pub bonus: i32,
}

/// Sum of all art-bonus effects (e.g. Puissant Art) targeting one Art. Arts are
/// not parameterized, so the target is matched by id alone. Two virtues boosting
/// the same Art stack.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4818-4820 (Puissant
/// Art, +3; may be taken twice, for two different Arts).
pub fn art_bonus(entity: &Entity, ruleset: &Ruleset, art: &Id) -> i32 {
    let mut bonus = 0;
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored bonus.
            match effect {
                Effect::ArtBonus { param, amount } if selection.params.get(param) == Some(art) => {
                    bonus += i32::from(*amount);
                }
                // Not an art bonus for this target; contributes nothing here.
                Effect::ArtBonus { .. }
                | Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. } => {}
            }
        }
    }
    bonus
}

/// The effective score of `art`: the highest bought score the entity holds for
/// it plus its bonus. An Art the entity has not bought counts as 0.
pub fn effective_art_score(entity: &Entity, ruleset: &Ruleset, art: &Id) -> i32 {
    let bought = entity
        .art_scores
        .iter()
        .filter(|a| &a.art == art)
        .map(|a| i32::from(a.score))
        .max()
        .unwrap_or(0);
    bought + art_bonus(entity, ruleset, art)
}

/// Non-zero art bonuses, one per bought Art, for the UI to add onto each
/// displayed bought score. Arts with no bonus are omitted. Order follows
/// `art_scores`.
pub fn art_bonuses(entity: &Entity, ruleset: &Ruleset) -> Vec<ArtBonus> {
    let mut out = Vec::new();
    for a in &entity.art_scores {
        let bonus = art_bonus(entity, ruleset, &a.art);
        if bonus != 0 {
            out.push(ArtBonus {
                art: a.art.clone(),
                bonus,
            });
        }
    }
    out
}

/// Net limit shift for `characteristic` from `CharacteristicLimit` effects whose
/// sign matches `raising`: the sum of positive amounts when `raising` is true
/// (Great Characteristic) or of negative amounts when false (Poor). Two Greats
/// for the same characteristic sum to +2; two Poors to −2.
fn characteristic_limit_shift(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
    raising: bool,
) -> i32 {
    let mut shift = 0;
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored shift.
            match effect {
                Effect::CharacteristicLimit { param, amount }
                    if (*amount > 0) == raising && *amount != 0 =>
                {
                    let target = selection
                        .params
                        .get(param)
                        .and_then(Characteristic::from_id);
                    if target == Some(characteristic) {
                        shift += i32::from(*amount);
                    }
                }
                // Wrong sign, or not a limit shift; contributes nothing here.
                Effect::CharacteristicLimit { .. }
                | Effect::AbilityBonus { .. }
                | Effect::ArtBonus { .. } => {}
            }
        }
    }
    shift
}

/// The highest base score `characteristic` may be bought to: the ruleset's base
/// cap (+3) raised by each Great (Characteristic) targeting it (+1 apiece),
/// clamped at the absolute effective ceiling (+5). Great Characteristic grants no
/// points — it only opens this headroom; the score must still be bought.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3987-3989.
pub fn characteristic_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let Some(rules) = ruleset.characteristic_rules() else {
        return 0;
    };
    let base_max = i32::from(rules.base_max_score().unwrap_or(0));
    let ceiling = i32::from(rules.effective_max_score().unwrap_or(0));
    (base_max + characteristic_limit_shift(entity, ruleset, characteristic, true)).min(ceiling)
}

/// The lowest base score `characteristic` may be bought to: the ruleset's base
/// floor (−3) lowered by each Poor (Characteristic) targeting it (−1 apiece),
/// clamped at the absolute effective floor (−5). Poor Characteristic grants no
/// points — it only opens this headroom; the score must still be sold down.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:6598-6600.
pub fn characteristic_floor(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let Some(rules) = ruleset.characteristic_rules() else {
        return 0;
    };
    let base_min = i32::from(rules.base_min_score().unwrap_or(0));
    let floor = i32::from(rules.effective_min_score().unwrap_or(0));
    (base_min + characteristic_limit_shift(entity, ruleset, characteristic, false)).max(floor)
}

/// Non-zero ability bonuses, one per bought ability *instance*, for the UI to add
/// onto each displayed bought score. A parameterized ability ((Area) Lore) yields
/// one entry per instance so a Puissant bonus attaches to exactly the targeted
/// row. Instances with no bonus are omitted. Order follows `ability_scores`.
pub fn ability_bonuses(entity: &Entity, ruleset: &Ruleset) -> Vec<AbilityBonus> {
    let mut out = Vec::new();
    for a in &entity.ability_scores {
        let bonus = ability_bonus(entity, ruleset, &a.ability, a.parameter.as_deref());
        if bonus != 0 {
            out.push(AbilityBonus {
                ability: a.ability.clone(),
                parameter: a.parameter.clone(),
                bonus,
            });
        }
    }
    out
}

/// The per-characteristic buy cap for all eight characteristics, keyed by
/// characteristic — the spinner ceiling the UI enforces (Great Characteristic
/// raises individual entries). Every characteristic has a cap, so none is
/// omitted.
pub fn characteristic_caps(entity: &Entity, ruleset: &Ruleset) -> BTreeMap<Characteristic, i32> {
    Characteristic::ALL
        .into_iter()
        .map(|c| (c, characteristic_cap(entity, ruleset, c)))
        .collect()
}

/// The per-characteristic buy floor for all eight characteristics, keyed by
/// characteristic — the spinner floor the UI enforces (Poor Characteristic
/// lowers individual entries). Every characteristic has a floor, so none is
/// omitted.
pub fn characteristic_floors(entity: &Entity, ruleset: &Ruleset) -> BTreeMap<Characteristic, i32> {
    Characteristic::ALL
        .into_iter()
        .map(|c| (c, characteristic_floor(entity, ruleset, c)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AbilityScore, ArtScore, EntityKind, RulesetRef, Selection};
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
            "magnitude": "free",
            "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.puissant_ability",
            "kind": "virtue",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }]
          },
          {
            "id": "virtue.great_characteristic",
            "kind": "virtue",
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
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }]
          }
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
        Ruleset::from_core_json_with_arts(
            "arm5-core",
            "2024.1",
            items,
            types,
            abilities,
            arts,
            characteristics,
        )
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
}
