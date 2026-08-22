//! Ability scoring: flat bonuses (Puissant Ability) and free starting-score
//! floors (Ability Score Grant). Split out of `effective.rs` (Viktor's V4
//! architecture finding — 93 free functions across 7 unrelated domains in one
//! file); pure code motion, no behavior change.

use super::*;

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
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
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
                // AbilityScoreGrant is a free *floor*, applied in
                // effective_ability_score, not an additive bonus.
                irrelevant_effect_variants!() => {}
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
    let floor = granted_ability_floor(entity, ruleset, ability, parameter);
    bought.max(floor) + ability_bonus(entity, ruleset, ability, parameter)
}

/// The highest free starting score granted to `ability` by any
/// [`Effect::AbilityScoreGrant`] (e.g. Second Sight seeding Second Sight 1). The
/// target is fixed by the granting virtue, so it matches by ability id. Granted
/// abilities are plain (single-instance), so only the parameter-less instance
/// receives the floor. Grants do not stack — a higher grant wins — so this is a
/// `max`, not a sum, and it costs no experience (see [`crate::validation`]).
pub(crate) fn granted_ability_floor(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> i32 {
    if parameter.is_some() {
        return 0;
    }
    let mut floor = 0;
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::AbilityScoreGrant {
                ability: granted,
                amount,
            } = effect
                && granted == ability
            {
                floor = floor.max(i32::from(*amount));
            }
        }
    }
    floor
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

/// A free starting-score floor a virtue grants to one ability (e.g. Second Sight
/// → Second Sight 1), for the frontend to show as the ability's effective score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityFloor {
    /// The granted ability's id.
    pub ability: Id,
    /// The free bought-score floor (the highest grant, if several apply).
    pub floor: i32,
}

/// Every ability granted a free starting score by an [`Effect::AbilityScoreGrant`],
/// each at its highest grant. Ordered by ability id (deduped), so the frontend can
/// show the floor as the ability's effective score without recomputing it.
pub fn ability_score_floors(entity: &Entity, ruleset: &Ruleset) -> Vec<AbilityFloor> {
    let mut floors: BTreeMap<Id, i32> = BTreeMap::new();
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::AbilityScoreGrant { ability, amount } = effect {
                let floor = floors.entry(ability.clone()).or_insert(0);
                *floor = (*floor).max(i32::from(*amount));
            }
        }
    }
    floors
        .into_iter()
        .map(|(ability, floor)| AbilityFloor { ability, floor })
        .collect()
}
