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
use crate::ruleset::Ruleset;
use crate::types::{Effect, Entity, Id, Selection};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, VecDeque};

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
pub fn selections_for_effects<'a>(entity: &'a Entity, ruleset: &Ruleset) -> Cow<'a, [Selection]> {
    let granted = crate::house::granted_selections(entity, ruleset);
    if granted.is_empty() {
        Cow::Borrowed(&entity.selections)
    } else {
        let mut combined = entity.selections.clone();
        combined.extend(granted);
        Cow::Owned(combined)
    }
}

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
                Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. } => {}
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
pub fn granted_ability_floor(
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
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
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
                | Effect::CharacteristicLimit { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. } => {}
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
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
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
                // CharacteristicPoints grants budget, not a range shift, and is
                // read by characteristic_points_granted.
                Effect::CharacteristicLimit { .. }
                | Effect::AbilityBonus { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. } => {}
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

/// The experience charged against a pool for a bought score whose advancement
/// table cost is `table_xp`, under an optional Affinity multiplier.
///
/// Affinity (Ability/Art) says creation XP "counts as" `num/den` of itself
/// (3/2, rounded up): so the points actually charged to reach a fixed table cost
/// `T` are the smallest `c` with `ceil(c·num/den) ≥ T`, which is
/// `ceil(T·den/num)`. The worked example (Perdo 10, Art table T=55, 3/2):
/// `ceil(55·2/3) = ceil(36.67) = 37`, which the rules say counts as 56 ≥ 55.
/// Integer-only so the engine stays exact.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3372-3378, worked
/// example `:2443`.
pub(crate) fn charged_cost(table_xp: u32, affinity: Option<(u8, u8)>) -> u32 {
    match affinity {
        Some((num, den)) if num != 0 => table_xp
            .saturating_mul(u32::from(den))
            .div_ceil(u32::from(num)),
        _ => table_xp,
    }
}

/// Of several Affinity multipliers on one target, the one giving the greatest
/// cost reduction (largest `den/num`). Affinities do not stack, so the single
/// most generous wins. Compares `d1/n1` vs `d2/n2` as `d1·n2` vs `d2·n1`.
fn best_affinity(multipliers: impl Iterator<Item = (u8, u8)>) -> Option<(u8, u8)> {
    multipliers.reduce(|a, b| {
        let (an, ad) = (u32::from(a.0), u32::from(a.1));
        let (bn, bd) = (u32::from(b.0), u32::from(b.1));
        if ad * bn >= bd * an { a } else { b }
    })
}

/// The Affinity multiplier applying to one ability instance, if any
/// ([`Effect::AffinityAbilityCost`] targeting it). Matches the instance exactly,
/// like [`ability_bonus`].
fn ability_affinity(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Option<(u8, u8)> {
    let instance_key = ruleset
        .abilities
        .get(ability)
        .and_then(|a| a.parameter.as_deref());
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            .filter_map(move |effect| match effect {
                Effect::AffinityAbilityCost {
                    param,
                    counts_as_num,
                    counts_as_den,
                } if selection.params.get(param) == Some(ability) => {
                    let matches = match instance_key {
                        None => true,
                        Some(key) => selection.params.get(key).map(Id::as_str) == parameter,
                    };
                    matches.then_some((*counts_as_num, *counts_as_den))
                }
                _ => None,
            })
    });
    best_affinity(found)
}

/// The Affinity multiplier applying to one Art, if any
/// ([`Effect::AffinityArtCost`] targeting it). Arts are matched by id alone.
fn art_affinity(entity: &Entity, ruleset: &Ruleset, art: &Id) -> Option<(u8, u8)> {
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            .filter_map(move |effect| match effect {
                Effect::AffinityArtCost {
                    param,
                    counts_as_num,
                    counts_as_den,
                } if selection.params.get(param) == Some(art) => {
                    Some((*counts_as_num, *counts_as_den))
                }
                _ => None,
            })
    });
    best_affinity(found)
}

/// A restricted experience pool granted by a virtue, with how much of it the
/// character's eligible spends actually consume (from the allocation). Serializes
/// for the frontend so the XP bar can show each pool's `used`/`amount`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedXpPool {
    /// Points granted to this pool.
    pub amount: u32,
    /// Points the allocation draws from this pool (≤ `amount`; remainder wasted).
    pub used: u32,
    /// Eligible ability ids (empty when eligibility is purely by category).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub abilities: Vec<Id>,
    /// Eligible ability categories (empty when eligibility is purely by id).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<AbilityCategory>,
}

/// The result of allocating Ability + Art spends across the general experience
/// pool and any restricted pools (Educated/Warrior/Privileged). Computed by a
/// max-flow feasibility solve; `total_demand > max_flow` means the spends cannot
/// all be funded (overspend by `total_demand - max_flow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XpAllocation {
    /// Sum of every spend's charged cost (post-Affinity).
    pub total_demand: u32,
    /// Maximum demand that can be funded. Equals `total_demand` iff legal.
    pub max_flow: u32,
    /// The general pool size (`Entity::xp_pool`).
    pub general_pool: u32,
    /// Points drawn from the general pool by the allocation.
    pub general_used: u32,
    /// The restricted pools with their consumed amounts.
    pub restricted: Vec<RestrictedXpPool>,
}

/// One bought score's funding demand for the flow solve.
struct Spend {
    cost: u32,
    /// The ability id + category, or `None` for an Art (Arts draw only from the
    /// general pool — no restricted grant covers them).
    ability: Option<(Id, AbilityCategory)>,
}

/// Whether a restricted pool may fund a spend: an ability whose id is listed or
/// whose category is listed. Arts are never eligible.
fn pool_covers(pool: &RestrictedXpPool, spend: &Spend) -> bool {
    match &spend.ability {
        Some((id, category)) => pool.abilities.contains(id) || pool.categories.contains(category),
        None => false,
    }
}

/// Allocates the entity's Ability + Art spends across the general pool and every
/// restricted pool, by max-flow feasibility. The general pool funds any spend;
/// each restricted pool funds only its eligible Abilities; overlapping
/// eligibility is resolved globally (greedy assignment would strand capacity).
/// A score the advancement table cannot price contributes 0 (already flagged by
/// `validate_abilities`/`validate_arts`).
pub fn xp_allocation(entity: &Entity, ruleset: &Ruleset) -> XpAllocation {
    // Spends: abilities (Affinity-reduced, with category for eligibility) + arts.
    let mut spends: Vec<Spend> = Vec::new();
    for a in &entity.ability_scores {
        let Some(table) = ruleset.advancement.xp_for_score(a.score) else {
            continue;
        };
        let cost = charged_cost(
            table,
            ability_affinity(entity, ruleset, &a.ability, a.parameter.as_deref()),
        );
        let ability = ruleset
            .abilities
            .get(&a.ability)
            .map(|def| (a.ability.clone(), def.category));
        spends.push(Spend { cost, ability });
    }
    for a in &entity.art_scores {
        let Some(table) = ruleset.art_advancement.xp_for_score(a.score) else {
            continue;
        };
        let cost = charged_cost(table, art_affinity(entity, ruleset, &a.art));
        spends.push(Spend {
            cost,
            ability: None,
        });
    }

    // Restricted pools, one node per RestrictedAbilityXp effect instance.
    let mut restricted: Vec<RestrictedXpPool> = Vec::new();
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::RestrictedAbilityXp {
                amount,
                abilities,
                categories,
            } = effect
            {
                restricted.push(RestrictedXpPool {
                    amount: *amount,
                    used: 0,
                    abilities: abilities.clone(),
                    categories: categories.clone(),
                });
            }
        }
    }

    let total_demand: u32 = spends.iter().map(|s| s.cost).sum();
    let general_pool = entity.xp_pool;

    // Flow graph: source(0) → sink(1); general(2) and restricted pools
    // (3..3+R) are pool nodes; spends follow. cap is the residual matrix.
    let r = restricted.len();
    let s = spends.len();
    let n = 3 + r + s;
    let general_node = 2;
    let pool_node = |i: usize| 3 + i;
    let spend_node = |j: usize| 3 + r + j;
    let (source, sink) = (0usize, 1usize);

    let mut cap = vec![vec![0u32; n]; n];
    cap[source][general_node] = general_pool;
    for (i, pool) in restricted.iter().enumerate() {
        cap[source][pool_node(i)] = pool.amount;
    }
    for (j, spend) in spends.iter().enumerate() {
        cap[spend_node(j)][sink] = spend.cost;
        // The general pool can fund any spend.
        cap[general_node][spend_node(j)] = spend.cost;
        for (i, pool) in restricted.iter().enumerate() {
            if pool_covers(pool, spend) {
                cap[pool_node(i)][spend_node(j)] = spend.cost;
            }
        }
    }

    let max_flow = max_flow(n, source, sink, &mut cap);

    // Residual on source→pool tells how much each pool funded.
    let general_used = general_pool - cap[source][general_node];
    for (i, pool) in restricted.iter_mut().enumerate() {
        pool.used = pool.amount - cap[source][pool_node(i)];
    }

    XpAllocation {
        total_demand,
        max_flow,
        general_pool,
        general_used,
        restricted,
    }
}

/// Edmonds-Karp max flow on a residual capacity matrix (BFS augmenting paths).
/// The graph is tiny (a few pools + a few dozen spends), so the simple matrix
/// form is more than fast enough.
fn max_flow(n: usize, source: usize, sink: usize, cap: &mut [Vec<u32>]) -> u32 {
    let mut total = 0;
    loop {
        let mut parent = vec![usize::MAX; n];
        parent[source] = source;
        let mut queue = VecDeque::new();
        queue.push_back(source);
        while let Some(u) = queue.pop_front() {
            for v in 0..n {
                if parent[v] == usize::MAX && cap[u][v] > 0 {
                    parent[v] = u;
                    queue.push_back(v);
                }
            }
        }
        if parent[sink] == usize::MAX {
            return total;
        }
        // Bottleneck along the found path.
        let mut bottleneck = u32::MAX;
        let mut v = sink;
        while v != source {
            let u = parent[v];
            bottleneck = bottleneck.min(cap[u][v]);
            v = u;
        }
        // Augment.
        let mut v = sink;
        while v != source {
            let u = parent[v];
            cap[u][v] -= bottleneck;
            cap[v][u] += bottleneck;
            v = u;
        }
        total += bottleneck;
    }
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

/// Total Characteristic-buy points granted by [`Effect::CharacteristicPoints`]
/// (Improved Characteristics, +3 each, stackable), summed across selections.
pub fn characteristic_points_granted(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0;
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::CharacteristicPoints { amount } = effect {
                total += u32::from(*amount);
            }
        }
    }
    total
}

/// The restricted XP pools an entity holds, with their consumed amounts (for the
/// frontend XP bar). Convenience wrapper over [`xp_allocation`].
pub fn restricted_xp_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<RestrictedXpPool> {
    xp_allocation(entity, ruleset).restricted
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

    // ---- Phase 3: Affinity, restricted pools, characteristic points, grants ----

    /// charged_cost is the inverse of "counts as num/den, rounded up". The book's
    /// worked example: Perdo 10 needs 55 on the Art table; with Affinity 3/2 you
    /// pay 37 (which counts as ceil(37·3/2)=56 ≥ 55). Source: Core Rules :2443.
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
            "kind": "virtue", "magnitude": "free", "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.affinity_ability",
            "kind": "virtue", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
            "effects": [{ "type": "affinity_ability_cost", "param": "ability", "counts_as_num": 3, "counts_as_den": 2 }]
          },
          {
            "id": "virtue.affinity_art",
            "kind": "virtue", "magnitude": "minor", "category": "hermetic",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "affinity_art_cost", "param": "art", "counts_as_num": 3, "counts_as_den": 2 }]
          },
          {
            "id": "virtue.educated",
            "kind": "virtue", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.latin", "ability.artes_liberales"] }]
          },
          {
            "id": "virtue.warrior",
            "kind": "virtue", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["martial"] }]
          },
          {
            "id": "virtue.privileged_upbringing",
            "kind": "virtue", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["general", "academic", "martial"] }]
          },
          {
            "id": "virtue.improved_characteristics",
            "kind": "virtue", "magnitude": "minor", "category": "general",
            "entity_kinds": ["character"],
            "effects": [{ "type": "characteristic_points", "amount": 3 }]
          },
          {
            "id": "virtue.second_sight",
            "kind": "virtue", "magnitude": "minor", "category": "supernatural",
            "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }]
          }
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
          { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
            "category": "special", "entity_kinds": ["character"] },
          { "id": "virtue.heartbeast", "kind": "virtue", "magnitude": "major",
            "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "ability_score_grant", "ability": "ability.heartbeast", "amount": 1 }] },
          { "id": "virtue.puissant_art", "kind": "virtue", "magnitude": "minor",
            "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }] }
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
            characteristics: None,
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
}
