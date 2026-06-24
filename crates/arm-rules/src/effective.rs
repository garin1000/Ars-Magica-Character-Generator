//! Effective scores: a character's *bought* ability/characteristic scores plus
//! the bonuses granted by score-boosting virtues (Puissant Ability +2, Great
//! Characteristic +1). Effective scores are always computed, never stored.
//!
//! The bonuses are data: each virtue declares [`Effect`]s in the ruleset, and
//! the target ability/characteristic is named by the selection's parameter
//! value. The engine hardcodes no virtue IDs.
//!
//! Source: `Ars Magica - Definitive Edition (Core Rules).md:4814-4816` (Puissant
//! Ability, +2), `:3987-3989` (Great Characteristic, +1 to no more than +5).

use crate::characteristics::Characteristic;
use crate::ruleset::Ruleset;
use crate::types::{Effect, Entity, Id};
use std::collections::BTreeMap;

/// Sum of all ability-bonus effects (e.g. Puissant Ability) targeting `ability`
/// across the entity's selections. Two virtues boosting the same ability stack.
pub fn ability_bonus(entity: &Entity, ruleset: &Ruleset, ability: &Id) -> i32 {
    let mut bonus = 0;
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::AbilityBonus { param, amount } = effect
                && selection.params.get(param) == Some(ability)
            {
                bonus += i32::from(*amount);
            }
        }
    }
    bonus
}

/// The effective score of `ability`: the highest bought score the entity holds
/// for it (a parameterized ability may appear more than once) plus its bonus.
/// An ability the entity has not bought counts as 0.
pub fn effective_ability_score(entity: &Entity, ruleset: &Ruleset, ability: &Id) -> i32 {
    let bought = entity
        .ability_scores
        .iter()
        .filter(|a| &a.ability == ability)
        .map(|a| i32::from(a.score))
        .max()
        .unwrap_or(0);
    bought + ability_bonus(entity, ruleset, ability)
}

/// Sum of all characteristic-bonus effects (e.g. Great Characteristic) targeting
/// `characteristic`. Two Great Characteristics for the same one stack to +2.
pub fn characteristic_bonus(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let mut bonus = 0;
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::CharacteristicBonus { param, amount, .. } = effect {
                let target = selection
                    .params
                    .get(param)
                    .and_then(Characteristic::from_id);
                if target == Some(characteristic) {
                    bonus += i32::from(*amount);
                }
            }
        }
    }
    bonus
}

/// The effective score of `characteristic`: the bought (point-buy) score plus
/// its bonus. An untouched characteristic counts as 0.
pub fn effective_characteristic(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let base = entity
        .characteristics
        .get(&characteristic)
        .copied()
        .map_or(0, i32::from);
    base + characteristic_bonus(entity, ruleset, characteristic)
}

/// Non-zero ability bonuses keyed by ability id, for the UI to add onto each
/// displayed bought score. Abilities with no bonus are omitted.
pub fn ability_bonuses(entity: &Entity, ruleset: &Ruleset) -> BTreeMap<Id, i32> {
    let mut out = BTreeMap::new();
    for a in &entity.ability_scores {
        let bonus = ability_bonus(entity, ruleset, &a.ability);
        if bonus != 0 {
            out.insert(a.ability.clone(), bonus);
        }
    }
    out
}

/// Non-zero characteristic bonuses keyed by characteristic, for the UI badge.
pub fn characteristic_bonuses(entity: &Entity, ruleset: &Ruleset) -> BTreeMap<Characteristic, i32> {
    let mut out = BTreeMap::new();
    for characteristic in Characteristic::ALL {
        let bonus = characteristic_bonus(entity, ruleset, characteristic);
        if bonus != 0 {
            out.insert(characteristic, bonus);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AbilityScore, EntityKind, RulesetRef, Selection};
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    /// A ruleset with Puissant Ability (+2 ability), Great Characteristic (+1
    /// characteristic, base >= 3, up to twice), and the canonical characteristic
    /// table with a +5 effective ceiling.
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
            "effects": [{ "type": "characteristic_bonus", "param": "characteristic", "amount": 1, "min_base": 3 }],
            "max_per_target": 2
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
          { "id": "ability.stealth", "category": "general" }
        ] }"#;
        let characteristics = r#"{
          "start_points": 7,
          "effective_max": 5,
          "costs": [
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 },
            { "score": 1, "cost": 1 }, { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 },
            { "score": -3, "cost": -6 }
          ]
        }"#;
        Ruleset::from_core_json(
            "arm5-core",
            "2024.1",
            items,
            types,
            abilities,
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

    fn great(characteristic: Characteristic) -> Selection {
        Selection::with_params(
            Id::new("virtue.great_characteristic"),
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
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.awareness")), 2);
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.awareness")),
            5
        );
    }

    #[test]
    fn ability_bonus_zero_for_non_targeted_ability() {
        let rs = ruleset();
        let e = entity(vec![puissant("ability.awareness")]);
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.stealth")), 0);
        // No bought score and no matching bonus => effective 0.
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new("ability.stealth")),
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
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.awareness")), 2);
        assert_eq!(ability_bonus(&e, &rs, &Id::new("ability.stealth")), 2);
    }

    #[test]
    fn great_characteristic_adds_one() {
        let rs = ruleset();
        let mut e = entity(vec![great(Characteristic::Str)]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
        assert_eq!(characteristic_bonus(&e, &rs, Characteristic::Str), 1);
        assert_eq!(effective_characteristic(&e, &rs, Characteristic::Str), 4);
    }

    #[test]
    fn great_characteristic_taken_twice_stacks_to_two() {
        let rs = ruleset();
        let mut e = entity(vec![great(Characteristic::Str), great(Characteristic::Str)]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
        assert_eq!(characteristic_bonus(&e, &rs, Characteristic::Str), 2);
        assert_eq!(effective_characteristic(&e, &rs, Characteristic::Str), 5);
    }

    #[test]
    fn characteristic_bonus_zero_for_non_targeted_characteristic() {
        let rs = ruleset();
        let mut e = entity(vec![great(Characteristic::Str)]);
        e.characteristics = BTreeMap::from([(Characteristic::Str, 3), (Characteristic::Qik, 2)]);
        assert_eq!(characteristic_bonus(&e, &rs, Characteristic::Qik), 0);
        assert_eq!(effective_characteristic(&e, &rs, Characteristic::Qik), 2);
    }

    #[test]
    fn bonus_maps_omit_zero_entries() {
        let rs = ruleset();
        let mut e = entity(vec![
            puissant("ability.awareness"),
            great(Characteristic::Str),
        ]);
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
        e.characteristics = BTreeMap::from([(Characteristic::Str, 3), (Characteristic::Qik, 2)]);

        let abilities = ability_bonuses(&e, &rs);
        assert_eq!(
            abilities,
            BTreeMap::from([(Id::new("ability.awareness"), 2)])
        );

        let chars = characteristic_bonuses(&e, &rs);
        assert_eq!(chars, BTreeMap::from([(Characteristic::Str, 1)]));
    }
}
