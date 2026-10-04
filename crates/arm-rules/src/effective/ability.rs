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
/// Source: ArMDE:4814-4816 ("You may
/// only take this Virtue once for a given Ability"; each (Area) Lore is a
/// distinct Ability).
pub fn ability_bonus(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> i32 {
    sum_of(&ability_bonus_contributions(
        entity, ruleset, ability, parameter,
    ))
}

/// Each additive ability-bonus effect targeting the `(ability, parameter)`
/// instance, by the item that carries it — the one fold [`ability_bonus`] sums,
/// so the total and its breakdown cannot drift.
fn ability_bonus_contributions(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Vec<ScoreSource> {
    // The instance-discriminator key for a parameterized ability ((Area) Lore →
    // "area"); `None` for a plain ability (a single instance, matched by id).
    let instance_key = ruleset
        .abilities
        .get(ability)
        .and_then(|a| a.parameter.as_deref());
    let mut sources = Vec::new();
    for_each_effect!(entity, ruleset, |selection, effect| {
        // Exhaustive match so adding an Effect variant is a compile error
        // here, not a silently-ignored bonus.
        match effect {
            Effect::AbilityBonus { param, amount }
                if selection
                    .params
                    .get(param)
                    .and_then(SelectionParamValue::as_single)
                    == Some(ability) =>
            {
                let matches = match instance_key {
                    None => true,
                    // The selection must name this instance; one that omits
                    // the instance key targets no parameterized instance at
                    // all.
                    Some(key) => match selection
                        .params
                        .get(key)
                        .and_then(SelectionParamValue::as_single)
                    {
                        Some(named) => Some(named.as_str()) == parameter,
                        None => false,
                    },
                };
                if matches {
                    add_score_source(&mut sources, &selection.item_ref, i32::from(*amount));
                }
            }
            // The gated-target sibling (Student of (Realm)'s +2 Lore): the
            // targets are a FIXED list on the effect itself, not read off the
            // selection's own free parameter, so each is checked by id/gate
            // directly rather than via `instance_key`'s selection lookup.
            // Guarded (like the arm above) so the same variant can also sit in
            // `irrelevant_effect_variants!`'s shared tail for the case this
            // guard fails.
            Effect::AbilityBonusGated { targets, amount }
                if targets.iter().any(|t| t.ability() == ability) =>
            {
                for target in targets {
                    if target.ability() != ability || !target.active_for(selection) {
                        continue;
                    }
                    // Interim plain-string shim, still NOT rewritten to design
                    // § 4's Bound/Link matching even after CV5: no shipped
                    // `AbilityBonusGated` target declares an `instance`
                    // restriction at all (Student of (Realm)'s targets are
                    // unscoped, gated only), so this branch is latent, not
                    // live, exactly like the C0 §3/§4.2 precedent this crate
                    // already tracks for a different gate. CV5 threaded §4's
                    // matching through `effective/xp.rs`'s `AbilityInstanceRef`/
                    // `AuthorizedAbility` call graph (ownership permission and
                    // XP-pool funding), which is where every shipped Bound/
                    // Link site actually lives — `ability_bonus`'s own
                    // `parameter: Option<&str>` signature (a plain match key,
                    // not the typed `AbilityParameterValue`) would need its own
                    // wider threading through every caller for a branch no
                    // rules data reaches; deferred until a book ships a gated
                    // `AbilityBonusGated` target with an `instance` restriction
                    // to actually exercise it.
                    let matches = match instance_key {
                        None => true,
                        Some(_) => target.resolved_instance(selection).as_deref() == parameter,
                    };
                    if matches {
                        add_score_source(&mut sources, &selection.item_ref, i32::from(*amount));
                    }
                }
            }
            // Not an ability bonus for this target; contributes nothing here.
            // AbilityScoreGrant is a free *floor*, applied in
            // effective_ability_score, not an additive bonus.
            irrelevant_effect_variants!() => {}
        }
    });
    sources
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
    let bought = bought_ability_score(entity, ability, parameter);
    let floor = granted_ability_floor(entity, ruleset, ability, parameter);
    bought.max(floor) + ability_bonus(entity, ruleset, ability, parameter)
}

/// The highest bought score the entity holds for the exact `(ability, parameter)`
/// instance (0 if unbought).
fn bought_ability_score(entity: &Entity, ability: &Id, parameter: Option<&str>) -> i32 {
    entity
        .ability_scores
        .iter()
        .filter(|a| {
            &a.ability == ability
                && a.parameter
                    .as_ref()
                    .and_then(AbilityParameterValue::match_key)
                    == parameter
        })
        .map(|a| i32::from(a.score))
        .max()
        .unwrap_or(0)
}

/// The highest free starting score granted to the `(ability, parameter)`
/// instance by any [`Effect::AbilityScoreGrant`] (e.g. Second Sight seeding
/// Second Sight 1 — fixed, unparameterized, so it only ever matches
/// `parameter: None`) or [`Effect::AbilityScoreGrantParam`] (F-63, Enchanting
/// Ability: the floor applies only at the ONE instance the grant's own
/// `instance` resolves to — `resolve_instance` is the same Bound/Literal/plain
/// resolution [`crate::effective::resolve_ability_refs`] uses, so this cannot
/// drift from how the identical grant is read for authorization). Grants do
/// not stack — a higher grant wins — so this is a `max`, not a sum, and it
/// costs no experience (see [`crate::validation`]).
pub(crate) fn granted_ability_floor(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> i32 {
    granted_ability_floor_source(entity, ruleset, ability, parameter)
        .map_or(0, |(_item, floor)| floor.max(0))
}

/// The winning grant behind [`granted_ability_floor`]: the item carrying the
/// highest grant for the instance, with that grant's amount. The first item to
/// reach the highest amount wins a tie, so the breakdown names one item only.
fn granted_ability_floor_source(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Option<(Id, i32)> {
    let mut best: Option<(Id, i32)> = None;
    let mut consider = |item: &Id, amount: i32| {
        if best.as_ref().is_none_or(|(_, floor)| amount > *floor) {
            best = Some((item.clone(), amount));
        }
    };
    for_each_effect!(entity, ruleset, |selection, effect| {
        match effect {
            Effect::AbilityScoreGrant {
                ability: granted,
                amount,
            } if granted == ability && parameter.is_none() => {
                consider(&selection.item_ref, i32::from(*amount));
            }
            Effect::AbilityScoreGrantParam {
                ability: granted,
                instance,
                amount,
            } if granted == ability => {
                let resolved = crate::effective::resolve_instance(
                    granted.clone(),
                    instance.as_ref(),
                    entity,
                    ruleset,
                    selection,
                );
                if resolved.parameter.as_deref() == parameter {
                    consider(&selection.item_ref, i32::from(*amount));
                }
            }
            _ => {}
        }
    });
    best
}

/// Non-zero ability bonuses, one per ability *instance*, for the UI to add onto
/// each displayed bought score. A parameterized ability ((Area) Lore) yields one
/// entry per instance so a Puissant bonus attaches to exactly the targeted row.
/// Instances with no bonus are omitted.
///
/// Iterates the **union** of the Ability catalogue and the entity's bought
/// instances, deduped on `(id, parameter)` — the same reasoning as
/// [`crate::art_bonuses`] (Issue 13), plus one wrinkle Arts do not have:
///
/// * Catalogue, not just `ability_scores` (Issue 17): a Puissant Ability applies
///   at 0 bought points, and the Abilities tab hangs its badge on a row, so
///   gating on bought rows hid the bonus entirely until the first point was
///   bought — the character had Puissant Magic Theory and nothing said so.
/// * Bought instances too, not just the catalogue: a catalogue entry carries
///   `parameter: None`, and [`ability_bonus`] matches `(ability, parameter)`
///   exactly, so iterating definitions alone would report 0 for Puissant
///   "(Area) Lore: Brandenburg" on a bought Brandenburg row.
///
/// A Puissant naming a parameterized ability therefore surfaces only once the
/// instance is bought: with no instance named the target is `(id, None)`, which
/// [`ability_bonus`] scores 0 by design, and a named-but-unbought instance is in
/// neither the catalogue nor `ability_scores`. Both cases are reported instead by
/// `ability_bonus_dangling_target` on the Abilities phase, which is where the fix
/// (buying the instance) lives.
///
/// Order: catalogue (id) order, then any bought instance the catalogue does not
/// already name, in `ability_scores` order.
pub fn ability_bonuses(entity: &Entity, ruleset: &Ruleset) -> Vec<AbilityBonus> {
    candidate_ability_instances(entity, ruleset)
        .into_iter()
        .filter_map(|(ability, parameter)| {
            let bonus = ability_bonus(entity, ruleset, ability, parameter);
            (bonus != 0).then(|| AbilityBonus {
                ability: ability.clone(),
                parameter: parameter.map(str::to_owned),
                bonus,
            })
        })
        .collect()
}

/// The ability instances [`ability_bonuses`] walks: the catalogue (each with no
/// parameter), then every bought instance the catalogue does not already name,
/// deduped on `(id, parameter)` (the reasoning is on [`ability_bonuses`]).
fn candidate_ability_instances<'a>(
    entity: &'a Entity,
    ruleset: &'a Ruleset,
) -> Vec<(&'a Id, Option<&'a str>)> {
    let mut seen: BTreeSet<(&Id, Option<&str>)> = BTreeSet::new();
    let mut instances: Vec<(&Id, Option<&str>)> = Vec::new();
    for ability in ruleset.abilities() {
        if seen.insert((&ability.id, None)) {
            instances.push((&ability.id, None));
        }
    }
    for score in &entity.ability_scores {
        let instance = (
            &score.ability,
            score
                .parameter
                .as_ref()
                .and_then(AbilityParameterValue::match_key),
        );
        if seen.insert(instance) {
            instances.push(instance);
        }
    }
    instances
}

/// The per-source breakdown of one ability instance's effective-over-bought
/// delta. Serializes as `{ "ability": "<id>", "sources": [ScoreSource] }`, with
/// `parameter` added only when present, like [`AbilityBonus`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityBonusSources {
    /// The ability's id.
    pub ability: Id,
    /// The instance discriminator, as in [`AbilityBonus::parameter`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// Each contributing item; the amounts sum to effective minus bought.
    pub sources: Vec<ScoreSource>,
}

/// Each item that moves the `(ability, parameter)` instance's effective score off
/// its bought score: the winning grant, for the part of its floor above the
/// bought score (Second Sight 1, ArMDE:4888-4890), and every additive bonus
/// (Puissant Ability, ArMDE:4814-4816). Sums to `effective - bought` by
/// construction, since [`effective_ability_score`] is `max(bought, floor) + bonus`
/// and `max(bought, floor) - bought` is `max(0, floor - bought)`.
fn ability_score_sources(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Vec<ScoreSource> {
    let bought = bought_ability_score(entity, ability, parameter);
    let mut sources = Vec::new();
    if let Some((grant, floor)) = granted_ability_floor_source(entity, ruleset, ability, parameter)
    {
        add_score_source(&mut sources, &grant, (floor - bought).max(0));
    }
    for bonus in ability_bonus_contributions(entity, ruleset, ability, parameter) {
        add_score_source(&mut sources, &bonus.source, bonus.amount);
    }
    sources.retain(|s| s.amount != 0);
    sources
}

/// The per-source breakdown behind every ability instance whose effective score
/// differs from its bought score — the condition the Abilities tab shows its
/// effective badge on — naming each contributing item with its signed share (I3,
/// try-out finding 15). Unlike [`ability_bonuses`] this includes an instance
/// moved by a granted floor alone, since the badge shows that too.
///
/// Instances: those [`ability_bonuses`] walks, plus every instance a grant names
/// ([`ability_score_floors`]), so a parameter-bound floor on an unbought
/// instance is covered. Order: that walk's order, floors last.
pub fn ability_bonus_sources(entity: &Entity, ruleset: &Ruleset) -> Vec<AbilityBonusSources> {
    let floors = ability_score_floors(entity, ruleset);
    let mut instances = candidate_ability_instances(entity, ruleset);
    for floor in &floors {
        let instance = (&floor.ability, floor.parameter.as_deref());
        if !instances.contains(&instance) {
            instances.push(instance);
        }
    }
    instances
        .into_iter()
        .filter_map(|(ability, parameter)| {
            let sources = ability_score_sources(entity, ruleset, ability, parameter);
            (sum_of(&sources) != 0).then(|| AbilityBonusSources {
                ability: ability.clone(),
                parameter: parameter.map(str::to_owned),
                sources,
            })
        })
        .collect()
}

/// A free starting-score floor a virtue grants to one ability (e.g. Second Sight
/// → Second Sight 1), for the frontend to show as the ability's effective score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityFloor {
    /// The granted ability's id.
    pub ability: Id,
    /// The instance this floor applies to (F-63, `Effect::AbilityScoreGrantParam`)
    /// — `None` for a plain, unparameterized grant
    /// ([`Effect::AbilityScoreGrant`]), matching [`AbilityBonus::parameter`]'s
    /// own reasoning: the UI badge must land on the right row, never on every
    /// instance of the ability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The free bought-score floor (the highest grant, if several apply).
    pub floor: i32,
}

/// Every ability granted a free starting score by an [`Effect::AbilityScoreGrant`]
/// or [`Effect::AbilityScoreGrantParam`] (F-63), each at its highest grant per
/// `(ability, parameter)` instance. Ordered by `(ability, parameter)` (deduped),
/// so the frontend can show the floor as the ability's effective score without
/// recomputing it.
pub fn ability_score_floors(entity: &Entity, ruleset: &Ruleset) -> Vec<AbilityFloor> {
    let mut floors: BTreeMap<(Id, Option<String>), i32> = BTreeMap::new();
    for_each_effect!(entity, ruleset, |selection, effect| {
        match effect {
            Effect::AbilityScoreGrant { ability, amount } => {
                let floor = floors.entry((ability.clone(), None)).or_insert(0);
                *floor = (*floor).max(i32::from(*amount));
            }
            Effect::AbilityScoreGrantParam {
                ability,
                instance,
                amount,
            } => {
                let resolved = crate::effective::resolve_instance(
                    ability.clone(),
                    instance.as_ref(),
                    entity,
                    ruleset,
                    selection,
                );
                let floor = floors
                    .entry((ability.clone(), resolved.parameter))
                    .or_insert(0);
                *floor = (*floor).max(i32::from(*amount));
            }
            _ => {}
        }
    });
    floors
        .into_iter()
        .map(|((ability, parameter), floor)| AbilityFloor {
            ability,
            parameter,
            floor,
        })
        .collect()
}
