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
    Effect, Entity, EntityTypeProfile, Id, ItemKind, Magnitude, MightScore, Realm, ReputationType,
    Selection, SpellSelection,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
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
                // M5/5b in-play effects: consumed by derived.rs (5i); they never
                // alter a creation-legality total, so they are no-ops here.
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
                // effective_art_score, not a flat per-effect bonus; no-op here.
                | Effect::ElementalMagic { .. } => {}
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

/// A non-zero free effective-score bonus targeting one Characteristic (Giant
/// Blood +1 Str/Sta, Dwarf −1). Serializes for the frontend as
/// `{ "characteristic": "str", "bonus": N }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacteristicBonus {
    /// The affected Characteristic.
    pub characteristic: Characteristic,
    /// The summed free bonus (may be negative).
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
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
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
                // M5/5b in-play effects: consumed by derived.rs (5i); they never
                // alter a creation-legality total, so they are no-ops here.
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
                // effective_art_score, not a flat per-effect bonus; no-op here.
                | Effect::ElementalMagic { .. } => {}
            }
        }
    }
    bonus
}

/// The highest whole bought score the entity holds for `art` (0 if unbought).
fn bought_art_score(entity: &Entity, art: &Id) -> u8 {
    entity
        .art_scores
        .iter()
        .filter(|a| &a.art == art)
        .map(|a| a.score)
        .max()
        .unwrap_or(0)
}

/// The set of elemental Form ids the entity's Elemental Magic marker pools over,
/// if it carries one ([`Effect::ElementalMagic`]). `None` for a character without
/// the Virtue — the overwhelmingly common case, so the redistribution path is
/// skipped entirely.
fn elemental_magic_forms(entity: &Entity, ruleset: &Ruleset) -> Option<BTreeSet<Id>> {
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::ElementalMagic { forms } = effect {
                return Some(forms.clone());
            }
        }
    }
    None
}

/// The **score-space** boost Elemental Magic confers on one elemental Form: 0 for
/// a non-elemental Art or an entity without the marker. Reconstructs each pooled
/// Form's table-XP from its bought score, gives `art` half (rounded up) of every
/// *other* pooled Form's XP, and inverts the sum back to a score — the delta over
/// the bought score is the boost.
///
/// This is an XP-space bonus, nonlinear in the bought score, so unlike every flat
/// [`Effect::ArtBonus`] it cannot be a single stored amount. Redistribution
/// operates on the table-XP of the *whole bought score* (storage keeps no raw
/// assigned XP), so leftover XP between score thresholds is not represented — see
/// RULES.md.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3731-3737 (21 XP → 11
/// bonus each: `ceil(21/2)`, so rounding is **up**).
fn elemental_form_bonus(entity: &Entity, ruleset: &Ruleset, art: &Id) -> i32 {
    let Some(forms) = elemental_magic_forms(entity, ruleset) else {
        return 0;
    };
    if !forms.contains(art) {
        return 0;
    }
    let table = &ruleset.art_advancement;
    let own_score = bought_art_score(entity, art);
    let own_xp = table.xp_for_score(own_score).unwrap_or(0);
    let mut bonus_xp = 0u32;
    for other in &forms {
        if other == art {
            continue;
        }
        let other_xp = table
            .xp_for_score(bought_art_score(entity, other))
            .unwrap_or(0);
        // Half, rounded up (Core:3731 worked example: 21 → 11).
        bonus_xp += other_xp.div_ceil(2);
    }
    let boosted = table.score_for_xp(own_xp + bonus_xp);
    i32::from(boosted) - i32::from(own_score)
}

/// The effective score of `art`: the highest bought score the entity holds for
/// it, plus any flat bonus (Puissant Art) and any Elemental Magic XP-space boost.
/// An Art the entity has not bought counts as 0.
pub fn effective_art_score(entity: &Entity, ruleset: &Ruleset, art: &Id) -> i32 {
    let bought = i32::from(bought_art_score(entity, art));
    bought + art_bonus(entity, ruleset, art) + elemental_form_bonus(entity, ruleset, art)
}

/// Non-zero art bonuses, one per Art, for the UI to add onto each displayed
/// bought score. Each is the full effective-over-bought delta — flat Puissant Art
/// *and* any Elemental Magic XP-space boost — so the UI surfaces the elemental
/// redistribution exactly like a Puissant bonus. Arts with no bonus are omitted.
///
/// Iterates the full Art catalogue, not just bought `art_scores`: a Puissant Art
/// (or an Elemental Magic form boost) applies even at 0 bought points, but the UI
/// drops an Art's row when its bought score hits 0, so gating on `art_scores`
/// would hide the badge until the first point is bought (Issue 13). Iterating the
/// catalogue also naturally dedupes any duplicate bought rows. Order follows the
/// ruleset's Art order.
pub fn art_bonuses(entity: &Entity, ruleset: &Ruleset) -> Vec<ArtBonus> {
    let mut out = Vec::new();
    for art in ruleset.arts() {
        let bonus = effective_art_score(entity, ruleset, &art.id)
            - i32::from(bought_art_score(entity, &art.id));
        if bonus != 0 {
            out.push(ArtBonus {
                art: art.id.clone(),
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
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
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
                // M5/5b in-play effects: consumed by derived.rs (5i); they never
                // alter a creation-legality total, so they are no-ops here.
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
                // effective_art_score, not a flat per-effect bonus; no-op here.
                | Effect::ElementalMagic { .. } => {}
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
/// cost reduction. Affinities do not stack, so the single most generous wins.
/// A score "counts as `num/den` of itself", charged `table·den/num`, so a larger
/// `num/den` is cheaper — the most generous is `max(num/den)`. Compares
/// `n1/d1` vs `n2/d2` as `n1·d2` vs `n2·d1` to stay in integer arithmetic.
fn best_affinity(multipliers: impl Iterator<Item = (u8, u8)>) -> Option<(u8, u8)> {
    multipliers.reduce(|a, b| {
        let (an, ad) = (u32::from(a.0), u32::from(a.1));
        let (bn, bd) = (u32::from(b.0), u32::from(b.1));
        if an * bd >= bn * ad { a } else { b }
    })
}

/// The Affinity multiplier applying to one ability instance, if any
/// ([`Effect::AffinityAbilityCost`] targeting it). Matches the instance exactly,
/// like [`ability_bonus`]. `pub(crate)` so the age-cap validator can read whether
/// an Ability carries an Affinity (which raises its age cap by +2, Core:3374).
pub(crate) fn ability_affinity(
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
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored cost reduction.
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
                // A group Affinity (Linguist) covers a fixed set of ability ids,
                // any instance — so it matches by id regardless of `parameter`.
                Effect::GroupAffinityCost {
                    abilities,
                    counts_as_num,
                    counts_as_den,
                } if abilities.contains(ability) => Some((*counts_as_num, *counts_as_den)),
                // Not an Affinity for this ability instance; no reduction here.
                Effect::AffinityAbilityCost { .. }
                | Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
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
                // M5/5b in-play effects: consumed by derived.rs (5i); not an
                // Affinity, so no cost reduction here.
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
                // Elemental Magic is an XP-space Art boost, not an Affinity/cost
                // reduction; no-op here.
                | Effect::ElementalMagic { .. } => None,
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
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-ignored cost reduction.
            .filter_map(move |effect| match effect {
                Effect::AffinityArtCost {
                    param,
                    counts_as_num,
                    counts_as_den,
                } if selection.params.get(param) == Some(art) => {
                    Some((*counts_as_num, *counts_as_den))
                }
                // Not an Affinity for this Art; no reduction here.
                Effect::AffinityArtCost { .. }
                | Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
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
                // M5/5b in-play effects: consumed by derived.rs (5i); not an
                // Affinity, so no cost reduction here.
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
                // Elemental Magic is an XP-space Art boost, not an Affinity/cost
                // reduction; no-op here.
                | Effect::ElementalMagic { .. } => None,
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
///
/// Serializes like its sibling result types (`RestrictedXpPool`, `AbilityBonus`,
/// `Balance`, …) so a Tauri command can hand the full allocation to the frontend
/// directly — including `max_flow`/`general_pool`, which let the UI surface the
/// overspend delta — rather than reshaping a subset of fields at the IPC edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// One bought score's funding demand for the flow solve, tagged with what it buys
/// so the kind-specific restricted pools know whether they may fund it.
struct Spend {
    cost: u32,
    kind: SpendKind,
}

/// What a [`Spend`] buys — decides which restricted pools may fund it (the general
/// pool always can). Ability spends draw RestrictedAbilityXp pools; Mastery spends
/// draw SpellMasteryXp pools; the two never cross, and Arts have no restricted pool.
enum SpendKind {
    /// An Ability score (id + category), eligible for RestrictedAbilityXp pools.
    Ability(Id, AbilityCategory),
    /// An Art score — funded from the general pool only.
    Art,
    /// A per-spell Spell Mastery Ability, eligible for SpellMasteryXp pools only.
    Mastery,
}

/// A restricted pool's funding scope for the flow solve.
enum PoolEligibility {
    /// An ability-XP grant (Educated/Warrior/Privileged): funds an Ability whose
    /// id is listed or whose category is listed. Never Arts, never Mastery.
    Ability {
        abilities: Vec<Id>,
        categories: Vec<AbilityCategory>,
    },
    /// A Spell-Mastery grant (Mastered Spells): funds only Spell Mastery spends.
    Mastery,
}

/// One restricted pool in the flow graph: its capacity and what it may fund.
struct FlowPool {
    amount: u32,
    eligibility: PoolEligibility,
}

/// Whether a restricted pool may fund a spend. Ability pools cover only Ability
/// spends they list (by id or category); Mastery pools cover only Mastery spends.
/// No pool covers an Art (general pool only), and the two pool kinds never cross.
fn pool_covers(eligibility: &PoolEligibility, spend: &Spend) -> bool {
    match (eligibility, &spend.kind) {
        (
            PoolEligibility::Ability {
                abilities,
                categories,
            },
            SpendKind::Ability(id, category),
        ) => abilities.contains(id) || categories.contains(category),
        (PoolEligibility::Mastery, SpendKind::Mastery) => true,
        // Every remaining combination is explicitly uncovered, so a new
        // PoolEligibility or SpendKind variant forces a decision here rather than
        // silently defaulting to false: Ability pools never fund Arts or Mastery,
        // and Mastery pools never fund Abilities or Arts.
        (PoolEligibility::Ability { .. }, SpendKind::Art)
        | (PoolEligibility::Ability { .. }, SpendKind::Mastery)
        | (PoolEligibility::Mastery, SpendKind::Ability(..))
        | (PoolEligibility::Mastery, SpendKind::Art) => false,
    }
}

/// Allocates the entity's Ability + Art spends across the general pool and every
/// restricted pool, by max-flow feasibility. The general pool funds any spend;
/// each restricted pool funds only its eligible Abilities; overlapping
/// eligibility is resolved globally (greedy assignment would strand capacity).
/// A score the advancement table cannot price contributes 0 (already flagged by
/// `validate_abilities`/`validate_arts`).
pub fn xp_allocation(entity: &Entity, ruleset: &Ruleset) -> XpAllocation {
    // Spends: abilities (Affinity-reduced, with category for eligibility) + arts +
    // per-spell Spell Mastery Abilities.
    let mut spends: Vec<Spend> = Vec::new();
    for a in &entity.ability_scores {
        let Some(table) = ruleset.advancement.xp_for_score(a.score) else {
            continue;
        };
        // A Virtue-granted Supernatural-Ability floor (e.g. Second Sight 1) is
        // free: the player "will not need to spend experience points for the
        // first point". So only the score above the granted floor is charged —
        // the floor's own table cost is subtracted before Affinity is applied.
        // Source: Ars Magica - Definitive Edition (Core Rules).md:2639.
        let floor = granted_ability_floor(entity, ruleset, &a.ability, a.parameter.as_deref());
        let floor_table = u8::try_from(floor)
            .ok()
            .filter(|f| *f > 0)
            .and_then(|f| ruleset.advancement.xp_for_score(f))
            .unwrap_or(0);
        let payable = table.saturating_sub(floor_table);
        let cost = charged_cost(
            payable,
            ability_affinity(entity, ruleset, &a.ability, a.parameter.as_deref()),
        );
        // A catalogue-known ability carries its category (for restricted-pool
        // eligibility); an unknown one funds from the general pool only, like an Art.
        let kind = ruleset
            .abilities
            .get(&a.ability)
            .map(|def| SpendKind::Ability(a.ability.clone(), def.category))
            .unwrap_or(SpendKind::Art);
        spends.push(Spend { cost, kind });
    }
    for a in &entity.art_scores {
        let Some(table) = ruleset.art_advancement.xp_for_score(a.score) else {
            continue;
        };
        let cost = charged_cost(table, art_affinity(entity, ruleset, &a.art));
        spends.push(Spend {
            cost,
            kind: SpendKind::Art,
        });
    }
    // Spell Mastery is an Ability (Core Rules.md:9516, :7143) bought from the
    // Ability advancement table (:15952, :15956-15979). Flawless Magic auto-masters
    // every spell at a free floor (charge only above it, like a granted Supernatural
    // floor) AND doubles all mastery Advancement Totals (an Affinity that halves the
    // charge). The mastery pool (Mastered Spells) — not the ability-restricted pools
    // — plus the general pool fund it.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:3887-3889, :4471-4474.
    let mastery_floor = spell_mastery_floor(entity, ruleset);
    let mastery_floor_table = if mastery_floor > 0 {
        ruleset.advancement.xp_for_score(mastery_floor).unwrap_or(0)
    } else {
        0
    };
    let mastery_affinity = spell_mastery_advancement_affinity(entity, ruleset);
    for spell in &entity.spells {
        let bought = spell.mastery.unwrap_or(0);
        if bought == 0 {
            continue;
        }
        let Some(table) = ruleset.advancement.xp_for_score(bought) else {
            continue;
        };
        let payable = table.saturating_sub(mastery_floor_table);
        let cost = charged_cost(payable, mastery_affinity);
        if cost == 0 {
            continue;
        }
        spends.push(Spend {
            cost,
            kind: SpendKind::Mastery,
        });
    }

    // Restricted pools: one per RestrictedAbilityXp effect (Educated/Warrior/…),
    // plus a single Spell-Mastery pool (Mastered Spells, summed). The mastery pool
    // is flow-only — it is not surfaced in `restricted`, which the UI reserves for
    // ability-XP grants.
    let mut flow_pools: Vec<FlowPool> = Vec::new();
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
                flow_pools.push(FlowPool {
                    amount: *amount,
                    eligibility: PoolEligibility::Ability {
                        abilities: abilities.clone(),
                        categories: categories.clone(),
                    },
                });
            }
        }
    }
    let mastery_pool = spell_mastery_xp(entity, ruleset);
    if mastery_pool > 0 {
        flow_pools.push(FlowPool {
            amount: mastery_pool,
            eligibility: PoolEligibility::Mastery,
        });
    }

    let total_demand: u32 = spends.iter().map(|s| s.cost).sum();
    // Skilled/Weak Parens (and any GeneralXp effect) adjust the apprenticeship
    // pool; a net-negative grant clamps at 0 rather than underflowing.
    let general_pool = clamp_to_u32(i64::from(entity.xp_pool) + general_xp_bonus(entity, ruleset));

    // Flow graph: source(0) → sink(1); general(2) and restricted pools
    // (3..3+R) are pool nodes; spends follow. cap is the residual matrix.
    let r = flow_pools.len();
    let s = spends.len();
    let n = 3 + r + s;
    let general_node = 2;
    let pool_node = |i: usize| 3 + i;
    let spend_node = |j: usize| 3 + r + j;
    let (source, sink) = (0usize, 1usize);

    let mut cap = vec![vec![0u32; n]; n];
    for (i, pool) in flow_pools.iter().enumerate() {
        cap[source][pool_node(i)] = pool.amount;
    }
    for (j, spend) in spends.iter().enumerate() {
        cap[spend_node(j)][sink] = spend.cost;
        // The general pool can fund any spend.
        cap[general_node][spend_node(j)] = spend.cost;
        for (i, pool) in flow_pools.iter().enumerate() {
            if pool_covers(&pool.eligibility, spend) {
                cap[pool_node(i)][spend_node(j)] = spend.cost;
            }
        }
    }

    // Two-phase fill on the shared residual matrix, so a spend the restricted
    // pools *can* cover drains them before the general pool (Educated/Warrior/
    // Privileged and Mastered-Spells XP is free-but-earmarked; the general pool
    // must stay available and no restricted XP wasted while eligible spends exist).
    // Phase 1: restricted-only max flow — the source→general edge stays closed.
    let restricted_flow = max_flow(n, source, sink, &mut cap);
    // Phase 2: open the source→general edge and continue Edmonds-Karp on the
    // same residuals. The sum is the true max flow with restricted usage
    // maximized, i.e. minimum general used.
    cap[source][general_node] = general_pool;
    let max_flow = restricted_flow + max_flow(n, source, sink, &mut cap);

    // Residual on source→pool tells how much each pool funded.
    let general_used = general_pool - cap[source][general_node];
    // Surface only the ability-XP pools (the mastery pool is accounted separately).
    let mut restricted: Vec<RestrictedXpPool> = Vec::new();
    for (i, pool) in flow_pools.iter().enumerate() {
        if let PoolEligibility::Ability {
            abilities,
            categories,
        } = &pool.eligibility
        {
            restricted.push(RestrictedXpPool {
                amount: pool.amount,
                used: pool.amount - cap[source][pool_node(i)],
                abilities: abilities.clone(),
                categories: categories.clone(),
            });
        }
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

/// Net Characteristic-buy points granted by [`Effect::CharacteristicPoints`],
/// summed across selections. Signed: Improved Characteristics adds +3 each, Weak
/// Characteristics subtracts 3 each; both stack, so the net may be negative.
pub fn characteristic_points_granted(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0;
    let selections = selections_for_effects(entity, ruleset);
    for selection in selections.iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::CharacteristicPoints { amount } = effect {
                total += i32::from(*amount);
            }
        }
    }
    total
}

/// The character's derived Size: base 0 plus every [`Effect::SizeDelta`]
/// (Large +1, Giant Blood +2, Small Frame −1, Dwarf −2), summed across
/// selections. Size is not a bought Characteristic — it has no cost and no buy
/// cap. Source: Core Rules.md:3975-3978, :4229-4231, :5996-5998, :6767-6769.
pub fn size(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::SizeDelta { amount } = effect {
                total += i32::from(*amount);
            }
        }
    }
    total
}

/// The free effective-score bonus a virtue/flaw grants to `characteristic`
/// ([`Effect::CharacteristicScoreDelta`], e.g. Giant Blood +1 Str/Sta), summed
/// across selections. Costs no buy points and stacks on top of the bought score.
pub fn characteristic_score_bonus(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let mut bonus = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::CharacteristicScoreDelta {
                characteristic: target,
                amount,
            } = effect
                && Characteristic::from_id(target) == Some(characteristic)
            {
                bonus += i32::from(*amount);
            }
        }
    }
    bonus
}

/// The effective score of `characteristic`: the bought score plus any free
/// [`Effect::CharacteristicScoreDelta`] bonus. The bonus may push the effective
/// score beyond the normal ±5 ceiling (Giant Blood's +1 reaches +6).
pub fn effective_characteristic_score(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let bought = entity
        .characteristics
        .get(&characteristic)
        .copied()
        .map_or(0, i32::from);
    bought + characteristic_score_bonus(entity, ruleset, characteristic)
}

/// Non-zero characteristic bonuses, one per affected Characteristic (canonical
/// order), for the UI to show alongside the bought score. Characteristics with
/// no bonus are omitted.
pub fn characteristic_bonuses(entity: &Entity, ruleset: &Ruleset) -> Vec<CharacteristicBonus> {
    Characteristic::ALL
        .into_iter()
        .filter_map(|c| {
            let bonus = characteristic_score_bonus(entity, ruleset, c);
            (bonus != 0).then_some(CharacteristicBonus {
                characteristic: c,
                bonus,
            })
        })
        .collect()
}

/// Effective Characteristic scores after aging drops AND free virtue deltas, one
/// entry per Characteristic whose effective value differs from its bought score
/// (canonical order). Characteristics unchanged from the bought score are omitted;
/// the UI falls back to the bought score for those. Surfacing this keeps the floor
/// clamp in [`effective_characteristic_after_aging`] as the single source of truth
/// (the UI never re-implements it).
pub fn effective_characteristics(
    entity: &Entity,
    ruleset: &Ruleset,
) -> BTreeMap<Characteristic, i32> {
    Characteristic::ALL
        .into_iter()
        .filter_map(|c| {
            let bought = entity.characteristics.get(&c).copied().map_or(0, i32::from);
            let effective = effective_characteristic_after_aging(entity, ruleset, c);
            (effective != bought).then_some((c, effective))
        })
        .collect()
}

/// Aging-drop counts per Characteristic (from [`aging_drops`]), only the non-zero
/// entries (canonical order), for the effective-score tooltip breakdown.
pub fn characteristic_aging_drops(entity: &Entity) -> BTreeMap<Characteristic, u32> {
    Characteristic::ALL
        .into_iter()
        .filter_map(|c| {
            let drops = aging_drops(entity, c);
            (drops != 0).then_some((c, drops))
        })
        .collect()
}

/// The restricted XP pools an entity holds, with their consumed amounts (for the
/// frontend XP bar). Convenience wrapper over [`xp_allocation`].
pub fn restricted_xp_pools(entity: &Entity, ruleset: &Ruleset) -> Vec<RestrictedXpPool> {
    xp_allocation(entity, ruleset).restricted
}

/// Clamps a signed budget total to a non-negative `u32` (a net-negative grant
/// floors at 0 rather than underflowing).
fn clamp_to_u32(n: i64) -> u32 {
    u32::try_from(n.max(0)).unwrap_or(u32::MAX)
}

/// Sums the [`Effect::SpellLevels`] amounts across the entity's selections (may
/// be negative; Skilled Parens +30, Weak Parens −30).
///
/// Surfaced on its own (not only folded into [`spell_levels_budget`]) so the
/// spell-levels bar can show the editable base beside a labelled V/F bonus,
/// mirroring how the XP bar lists extra pools beside the general one.
pub fn spell_levels_bonus(entity: &Entity, ruleset: &Ruleset) -> i64 {
    sum_signed_effect(entity, ruleset, |e| match e {
        Effect::SpellLevels { amount } => Some(*amount),
        // Exhaustive so adding an Effect variant is a compile error here, not a
        // silently-ignored contribution to the spell-levels budget.
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicLimit { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        | Effect::GeneralXp { .. }
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
        | Effect::ElementalMagic { .. } => None,
    })
}

/// Sums the [`Effect::GeneralXp`] amounts across the entity's selections (may be
/// negative; Skilled Parens +60, Weak Parens −60).
fn general_xp_bonus(entity: &Entity, ruleset: &Ruleset) -> i64 {
    sum_signed_effect(entity, ruleset, |e| match e {
        Effect::GeneralXp { amount } => Some(*amount),
        // Exhaustive so adding an Effect variant is a compile error here, not a
        // silently-ignored contribution to the general XP pool.
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicLimit { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        | Effect::SpellLevels { .. }
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
        | Effect::ElementalMagic { .. } => None,
    })
}

/// Sums a signed per-selection effect amount across everything that feeds the
/// effective layer (selections + derived grants).
fn sum_signed_effect(
    entity: &Entity,
    ruleset: &Ruleset,
    pick: impl Fn(&Effect) -> Option<i16>,
) -> i64 {
    let mut total: i64 = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Some(amount) = pick(effect) {
                total += i64::from(amount);
            }
        }
    }
    total
}

/// The base spell-levels budget BEFORE Skilled/Weak Parens modifiers: the
/// per-character [`Entity::spell_levels_override`] when set, otherwise the type
/// profile's `spell_levels` (120 for a magus; 0 for a type with no profile).
/// Factored so the effective payload and the validator select the base
/// identically and can never diverge (Issue 11).
// Source: Ars Magica - Definitive Edition (Core Rules).md:2215-2216, :2435
pub fn spell_levels_base(entity: &Entity, profile: Option<&EntityTypeProfile>) -> u32 {
    entity
        .spell_levels_override
        .unwrap_or_else(|| profile.map(|p| p.spell_levels).unwrap_or(0))
}

/// The magus's effective spell-levels budget: the base ([`spell_levels_base`])
/// plus any [`Effect::SpellLevels`] modifiers, clamped at 0.
pub fn spell_levels_budget(base: u32, entity: &Entity, ruleset: &Ruleset) -> u32 {
    clamp_to_u32(i64::from(base) + spell_levels_bonus(entity, ruleset))
}

/// The maximum level a magus may learn of a spell of the given Technique/Form:
/// the sum of Technique, Form, Intelligence, Magic Theory and 3 (Core:2465),
/// using effective Art/Ability scores. Returns an `i64` (small or negative for a
/// beginning magus). Requisite-Art reduction is a lab-total nuance out of scope.
/// Single source of truth: both the validation cap and the UI-surfaced cap read
/// this, so the two can never diverge.
// Source: Ars Magica - Definitive Edition (Core Rules).md:2465
pub fn spell_level_cap(entity: &Entity, ruleset: &Ruleset, technique: &Id, form: &Id) -> i64 {
    let tech = i64::from(effective_art_score(entity, ruleset, technique));
    let form = i64::from(effective_art_score(entity, ruleset, form));
    let int = i64::from(
        entity
            .characteristics
            .get(&Characteristic::Int)
            .copied()
            .unwrap_or(0),
    );
    let magic_theory = i64::from(effective_ability_score(
        entity,
        ruleset,
        &Id::new(crate::ruleset::ID_MAGIC_THEORY),
        None,
    ));
    tech + form + int + magic_theory + 3
}

/// A per-Technique/Form spell-level cap, surfaced to the frontend so the spell
/// picker can grey a spell whose level exceeds the magus's cap without
/// recomputing the derivation in JS. Serializes as
/// `{ "technique": "<id>", "form": "<id>", "cap": N }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellLevelCap {
    /// The Technique-class Art id (e.g. `art.creo`).
    pub technique: Id,
    /// The Form-class Art id (e.g. `art.ignem`).
    pub form: Id,
    /// The maximum learnable level for this Te/Fo combination (may be negative
    /// for a beginning magus).
    pub cap: i64,
}

/// The [`spell_level_cap`] for every Technique × Form combination in the Art
/// catalogue, sorted canonically by `(technique, form)`. The picker keys these
/// by the pair to look up a candidate spell's cap. One entry per combo (a spell's
/// cap depends only on its Te/Fo, never its level).
pub fn spell_level_caps(entity: &Entity, ruleset: &Ruleset) -> Vec<SpellLevelCap> {
    // `art_ids_of` guarantees the sort the canonical (technique, form) order needs.
    let techniques = ruleset.art_ids_of(crate::art::ArtType::Technique);
    let forms = ruleset.art_ids_of(crate::art::ArtType::Form);
    let mut caps = Vec::with_capacity(techniques.len() * forms.len());
    for technique in &techniques {
        for form in &forms {
            caps.push(SpellLevelCap {
                technique: technique.clone(),
                form: form.clone(),
                cap: spell_level_cap(entity, ruleset, technique, form),
            });
        }
    }
    caps
}

/// The learned level of a chosen spell: the catalogue's fixed level, or — for a
/// **General** spell — the per-character chosen level. `None` if the spell is
/// unknown to the catalogue, or a General spell has no chosen level yet.
pub fn resolved_spell_level(sel: &SpellSelection, ruleset: &Ruleset) -> Option<u32> {
    let spell = ruleset.spell(&sel.spell)?;
    match spell.level {
        Some(fixed) => Some(u32::from(fixed)),
        None => sel.level.map(u32::from),
    }
}

/// The character's Spell-Mastery XP pool: the sum of every
/// [`Effect::SpellMasteryXp`] (Mastered Spells +50, stackable). A restricted pool
/// spent only on per-spell Spell Mastery Abilities. Source: Core Rules.md:4471-4474.
pub fn spell_mastery_xp(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::SpellMasteryXp { amount } = effect {
                total += u32::from(*amount);
            }
        }
    }
    total
}

/// The mastery-score floor every known spell receives from
/// [`Effect::GrantsSpellMastery`] (Flawless Magic → 1). The highest floor wins.
/// Source: Core Rules.md:3887-3889.
pub fn spell_mastery_floor(entity: &Entity, ruleset: &Ruleset) -> u8 {
    let mut floor = 0u8;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::GrantsSpellMastery { score, .. } = effect {
                floor = floor.max(*score);
            }
        }
    }
    floor
}

/// The Advancement-Total multiplier applying to *every* Spell Mastery Ability, as
/// an Affinity "counts as num/den of itself" ([`Effect::GrantsSpellMastery`]'s
/// doubling: Flawless Magic → `(2, 1)`, halving the XP charged). `None` when no
/// grant reduces the cost. The most generous multiplier wins, like any Affinity.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3889.
pub fn spell_mastery_advancement_affinity(entity: &Entity, ruleset: &Ruleset) -> Option<(u8, u8)> {
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            .filter_map(|effect| match effect {
                Effect::GrantsSpellMastery {
                    advancement_num,
                    advancement_den,
                    ..
                    // A larger num/den is a genuine reduction; the identity 1/1
                    // (a plain floor grant) contributes no Affinity.
                } if u32::from(*advancement_num) > u32::from(*advancement_den) => {
                    Some((*advancement_num, *advancement_den))
                }
                // A plain floor grant (identity multiplier) or any other effect
                // contributes no advancement Affinity. Enumerated so a new Effect
                // variant is a compile error here until it is classified.
                Effect::GrantsSpellMastery { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
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
                | Effect::ElementalMagic { .. } => None,
            })
    });
    best_affinity(found)
}

/// The effective Spell Mastery score of one chosen spell: the higher of its
/// bought mastery and the granted floor (Flawless Magic auto-masters at 1).
pub fn effective_spell_mastery(sel: &SpellSelection, entity: &Entity, ruleset: &Ruleset) -> u8 {
    sel.mastery
        .unwrap_or(0)
        .max(spell_mastery_floor(entity, ruleset))
}

/// Total spell levels the entity's chosen spells consume. Unresolved General
/// spells (no chosen level) and unknown spells contribute 0.
pub fn spell_levels_used(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .spells
        .iter()
        .filter_map(|s| resolved_spell_level(s, ruleset))
        .sum()
}

// --- Phase 7: Gift/Supernatural, Confidence, Reputations, age cap ---

/// Whether the entity "has The Gift" per its type profile: a selection matching
/// the profile's `gift_id`, or one whose item category is in `gift_categories`.
/// Shared with `validate_gift_policy` so both use one definition.
pub(crate) fn has_the_gift(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: &EntityTypeProfile,
) -> bool {
    let by_id = profile
        .gift_id
        .as_ref()
        .is_some_and(|gid| entity.selections.iter().any(|s| &s.item_ref == gid));
    let by_category = !profile.gift_categories.is_empty()
        && entity.selections.iter().any(|s| {
            ruleset
                .point_items
                .get(&s.item_ref)
                .is_some_and(|item| profile.gift_categories.contains(&item.category))
        });
    by_id || by_category
}

/// A character's effective Confidence: the derived Confidence Score and the
/// Confidence Points backing it. Serializes like its sibling result types
/// (`Balance`, `AbilityBonus`) as `{ "score": N, "points": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    /// The Confidence Score (spent per die roll).
    pub score: u8,
    /// The Confidence Points available to refresh the score.
    pub points: u8,
}

/// The character's effective Confidence: the type profile's base plus every
/// [`Effect::ConfidenceBonus`], clamped at 0. Confidence is derived, never
/// stored. Source: Core Rules.md:2520-2526, 4900-4902.
pub fn confidence(
    base_score: u8,
    base_points: u8,
    entity: &Entity,
    ruleset: &Ruleset,
) -> Confidence {
    let mut score = i32::from(base_score);
    let mut points = i32::from(base_points);
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::ConfidenceBonus {
                score: s,
                points: p,
            } = effect
            {
                score += i32::from(*s);
                points += i32::from(*p);
            }
        }
    }
    let clamp = |n: i32| u8::try_from(n.max(0)).unwrap_or(u8::MAX);
    Confidence {
        score: clamp(score),
        points: clamp(points),
    }
}

/// The character's derived enchanted-device level budget: base 0 plus every
/// [`Effect::ItemLevelBudget`] (Magic Items +25, Redcap 50), summed. Source:
/// Core Rules.md:4347-4349, :4842-4846.
pub fn item_level_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::ItemLevelBudget { amount } = effect {
                total += u32::from(*amount);
            }
        }
    }
    total
}

/// The total enchanted-device level the entity's `devices` consume — the "used"
/// side of the item-level budget bar. Summed across every device. Source: Core
/// Rules.md:4347-4349.
pub fn item_level_used(entity: &Entity) -> u32 {
    entity.devices.iter().map(|d| u32::from(d.level)).sum()
}

/// The character's derived power-levels budget: base 0 plus every
/// [`Effect::PowerLevels`] grant (Demonic Blood 30, Demonic Powers +20, Strong
/// Angelic Heritage 30), summed. The being's `powers` are charged against it,
/// mirroring [`item_level_budget`]. Source: Realms of Power - The Infernal.md:4122,
/// :4142; The Divine (Revised).md:1977.
pub fn power_levels_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::PowerLevels { amount } = effect {
                total += u32::from(*amount);
            }
        }
    }
    total
}

/// The total power level the being's `powers` consume — the "used" side of the
/// power-levels budget bar. Source: Realms of Power - The Infernal.md:4122.
pub fn powers_used(entity: &Entity) -> u32 {
    entity.powers.iter().map(|p| u32::from(p.level)).sum()
}

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
/// Infernal Might 5, Demonic Might +2 → effective 7. Source: Realms of Power -
/// Magic.md:1470-1472; The Infernal.md:4120, :4136.
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
/// `u8`. Derived, never stored. Source: Core Rules.md:5169-5171.
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

/// The Warping Points granted by [`Effect::WarpingGrant`] (Warped by Magic → 5),
/// summed across selections and derived grants. The grant's declared *score* field
/// is **not** read here — the Warping Score is derived by inverting the advancement
/// curve over the point total (see [`warping_score`]), so the score is computed
/// from points alone and the two can never disagree.
fn warping_grant_points_in(selections: &[Selection], ruleset: &Ruleset) -> u32 {
    let mut points = 0u32;
    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::WarpingGrant {
                score: _,
                points: p,
            } = effect
            {
                points += u32::from(*p);
            }
        }
    }
    points
}

/// The character's total Warping Points: the stored [`Entity::warping_points`] plus
/// every grant-derived point across the full effect selection list. The single
/// point total the Warping Score is derived from, so stored and granted points can
/// never be double-counted or diverge. Owed warping fills carrying
/// [`Effect::WarpingGrant`] are filtered out of the folded grants (see
/// [`warping_granted_selections`]), so they never contribute here either.
/// Source: Core Rules.md:16464-16475.
pub fn warping_points_total(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .warping_points
        .saturating_add(warping_grant_points_in(
            selections_for_effects(entity, ruleset).as_ref(),
            ruleset,
        ))
}

/// The Warping Points that DETERMINE how many V/F are owed from Warping: the
/// stored points plus grant points from bought selections and non-warping grants
/// ([`entity_grants_base`]) ONLY. The owed warping fills are deliberately excluded
/// so a fill can never raise the score that decides how many fills are owed — the
/// recursion guard against the self-amplifying `warped_by_magic` feedback loop.
/// Source: Core Rules.md:16553-16561.
fn warping_points_for_owed(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut base = entity.selections.clone();
    base.extend(entity_grants_base(entity, ruleset));
    entity
        .warping_points
        .saturating_add(warping_grant_points_in(&base, ruleset))
}

/// The Warping Score used to decide the owed warping V/F: [`warping_points_for_owed`]
/// inverted through the advancement curve (owed fills excluded — the recursion
/// guard). Source: Core Rules.md:16553-16561.
fn warping_score_for_owed(entity: &Entity, ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement
        .score_for_xp(warping_points_for_owed(entity, ruleset))
}

/// The character's derived Warping Score: [`warping_points_total`] inverted through
/// the (Ability) advancement curve (Warping rises "like an Ability": cumulative
/// 5/15/30/50/75, so 15 points → Warping Score 2). Source: Core Rules.md:16464-16475.
pub fn warping_score(entity: &Entity, ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement
        .score_for_xp(warping_points_total(entity, ruleset))
}

/// A character's derived Warping: the Warping Score and the Warping Points it is
/// derived from. Serializes like its sibling result types as
/// `{ "score": N, "points": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warping {
    /// The Warping Score (advancement curve inverted over the point total).
    pub score: u8,
    /// The total accrued Warping Points (stored plus V/F grants).
    pub points: u32,
}

/// The character's derived Warping — the unified readout: the score from
/// [`warping_score`], the points from [`warping_points_total`]. Derived, never
/// stored on the entity as a resolved value. Source: Core Rules.md:7019-7021,
/// :16464-16475.
pub fn warping(entity: &Entity, ruleset: &Ruleset) -> Warping {
    Warping {
        score: warping_score(entity, ruleset),
        points: warping_points_total(entity, ruleset),
    }
}

/// The category slug a warping-owed supernatural Minor Virtue must belong to
/// (Core Rules.md:16559, "a supernatural Minor Virtue"). The category taxonomy is
/// data; this names the slug the rule's "supernatural" wording maps to.
const WARPING_SUPERNATURAL_CATEGORY: &str = "supernatural";

/// Stable `choice_key` prefix for each owed Minor Flaw slot (`…0`, `…1`).
pub(crate) const WARPING_MINOR_FLAW_KEY: &str = "warping.minor_flaw.";
/// Stable `choice_key` prefix for the owed supernatural Minor Virtue slot.
pub(crate) const WARPING_SUPERNATURAL_VIRTUE_KEY: &str = "warping.supernatural_virtue.";
/// Stable `choice_key` prefix for each owed Major Flaw slot.
pub(crate) const WARPING_MAJOR_FLAW_KEY: &str = "warping.major_flaw.";

/// The Virtues and Flaws a character owes from its Warping Score, per "Effects of
/// Warping" (Core Rules.md:16547-16561). These are auto-granted, off-budget V/F
/// (never counted against the creation Virtue/Flaw budget), filled by the player
/// choosing specific items (stored in [`Entity::warping_choices`]). Derived, never
/// stored as a resolved value.
///
/// Hermetic magi are exempt: Warping makes them prone to Wizard's Twilight
/// instead ("This replaces the normal effects", :16551), which this slice does
/// NOT model — a magus always owes zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WarpingOwed {
    /// Owed Minor Flaws: 1 at Warping Score 1 (:16553), 2 at Score 3 (:16557).
    pub minor_flaws: u8,
    /// Owed supernatural Minor Virtues: 1 at Warping Score 5 (:16559), else 0.
    pub minor_supernatural_virtues: u8,
    /// Owed Major Flaws: 1 at Warping Score 6 and every point thereafter (:16561).
    pub major_flaws: u8,
}

impl WarpingOwed {
    /// The owed V/F for a non-magus at Warping Score `score` — the pure threshold
    /// curve of "Effects of Warping". Source: Core Rules.md:16553-16561.
    pub fn from_score(score: u8) -> Self {
        WarpingOwed {
            // A Minor Flaw at Warping Score 1 (:16553); a second at Score 3 (:16557).
            minor_flaws: if score >= 3 {
                2
            } else if score >= 1 {
                1
            } else {
                0
            },
            // A supernatural Minor Virtue at Warping Score 5 (:16559).
            minor_supernatural_virtues: u8::from(score >= 5),
            // A Major Flaw at Warping Score 6, and every point thereafter (:16561).
            major_flaws: score.saturating_sub(5),
        }
    }
}

/// The Virtues/Flaws `entity` owes from Warping. Non-magi owe per the score
/// (derived via the recursion-guarded [`warping_score_for_owed`], so owed fills
/// never inflate the count); Hermetic magi (`profile.is_magus`) are exempt and
/// owe zero — Warping gives them Wizard's Twilight instead (Core Rules.md:16551).
pub fn warping_owed(entity: &Entity, ruleset: &Ruleset) -> WarpingOwed {
    if ruleset
        .profile(&entity.type_id)
        .is_some_and(|profile| profile.is_magus)
    {
        return WarpingOwed::default();
    }
    WarpingOwed::from_score(warping_score_for_owed(entity, ruleset))
}

/// Whether `item_ref` carries an [`Effect::WarpingGrant`]. Such an item is
/// INELIGIBLE as a warping-owed fill: folding its granted Warping Points back
/// into the score would self-amplify the owed count. The recursion guard rejects
/// it in validation and drops it in [`warping_granted_selections`].
pub(crate) fn item_carries_warping_grant(item_ref: &Id, ruleset: &Ruleset) -> bool {
    ruleset.point_items.get(item_ref).is_some_and(|item| {
        item.effects
            .iter()
            .any(|effect| matches!(effect, Effect::WarpingGrant { .. }))
    })
}

/// The owed warping V/F expressed as OPEN [`Grant`]s — one grant per owed slot,
/// each with a stable `choice_key` and the [`GrantConstraint`] its fill must
/// satisfy (a Minor Flaw, a supernatural Minor Virtue, or a Major Flaw). The list
/// length tracks the recursion-guarded Warping Score via [`warping_owed`]. The
/// frontend renders one picker per grant; validation resolves each pick against
/// its constraint. Source: Core Rules.md:16553-16561.
pub fn warping_owed_grants(entity: &Entity, ruleset: &Ruleset) -> Vec<Grant> {
    let owed = warping_owed(entity, ruleset);
    let mut grants = Vec::new();
    for i in 0..owed.minor_flaws {
        grants.push(warping_open_grant(
            format!("{WARPING_MINOR_FLAW_KEY}{i}"),
            ItemKind::Flaw,
            Magnitude::Minor,
            false,
        ));
    }
    for i in 0..owed.minor_supernatural_virtues {
        grants.push(warping_open_grant(
            format!("{WARPING_SUPERNATURAL_VIRTUE_KEY}{i}"),
            ItemKind::Virtue,
            Magnitude::Minor,
            true,
        ));
    }
    for i in 0..owed.major_flaws {
        grants.push(warping_open_grant(
            format!("{WARPING_MAJOR_FLAW_KEY}{i}"),
            ItemKind::Flaw,
            Magnitude::Major,
            false,
        ));
    }
    grants
}

/// Builds one owed-warping OPEN grant with the given key/kind/magnitude, adding
/// the supernatural category requirement for the Minor Virtue slot.
fn warping_open_grant(
    choice_key: String,
    kind: ItemKind,
    magnitude: Magnitude,
    supernatural: bool,
) -> Grant {
    let mut require_categories = BTreeSet::new();
    if supernatural {
        require_categories.insert(WARPING_SUPERNATURAL_CATEGORY.to_string());
    }
    Grant::Open {
        choice_key,
        constraint: GrantConstraint {
            kind,
            magnitude: Some(magnitude),
            require_categories,
            forbid_categories: BTreeSet::new(),
        },
    }
}

/// The off-budget owed warping V/F fills the player has chosen, resolved to real
/// [`Selection`]s so they fold through [`entity_grants`] for prereq/effect
/// purposes. Budget- and cap-exempt, exactly like House grants. A pick carrying
/// [`Effect::WarpingGrant`] is dropped (ineligible — the recursion guard), so a
/// warping fill can never feed Warping Points back into the owed count.
/// Source: Core Rules.md:16553-16561.
pub fn warping_granted_selections(entity: &Entity, ruleset: &Ruleset) -> Vec<Selection> {
    resolve_grants(
        &warping_owed_grants(entity, ruleset),
        &entity.warping_choices,
    )
    .into_iter()
    .filter(|selection| !item_carries_warping_grant(&selection.item_ref, ruleset))
    .collect()
}

/// The character's total accrued aging points across every Characteristic — the
/// character's Decrepitude XP (every aging point is 1 XP toward Decrepitude).
/// Source: Core Rules.md:16617.
pub fn decrepitude_points_total(entity: &Entity) -> u32 {
    entity.aging_points.values().map(|p| u32::from(*p)).sum()
}

/// The character's derived Decrepitude Score: [`decrepitude_points_total`] inverted
/// through the (Ability) advancement curve (Decrepitude rises "like an Ability",
/// 5×new score, so 17 aging points → Decrepitude 2). Source: Core Rules.md:16617.
pub fn decrepitude_score(entity: &Entity, ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement
        .score_for_xp(decrepitude_points_total(entity))
}

/// The number of Characteristic drops the accrued aging points force, DERIVED
/// from [`Entity::aging_points`] (never stored). Per the rule, once a
/// Characteristic's accrued points *exceed* the absolute value of its (already
/// aged-down) score it drops by one and its aging points reset. Simulated over
/// the lifetime point total: each drop consumes `|score| + 1` points and lowers
/// the score by one, so the threshold shrinks toward 0 and then grows again.
/// Worked examples: a Communication of +2 drops on its 3rd aging point; a
/// Stamina of −3 on its 4th.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16579, :16613.
///
/// Crate-internal primitive: the frontend consumes the surfaced
/// [`characteristic_aging_drops`] map (which wraps this per-Characteristic), so
/// this single-Characteristic query is not part of the curated public API.
pub(crate) fn aging_drops(entity: &Entity, characteristic: Characteristic) -> u32 {
    let bought = entity
        .characteristics
        .get(&characteristic)
        .copied()
        .map_or(0i64, i64::from);
    let mut remaining = entity
        .aging_points
        .get(&characteristic)
        .copied()
        .map_or(0u32, u32::from);
    let mut drops = 0u32;
    loop {
        let aged = bought - i64::from(drops);
        let threshold = u32::try_from(aged.unsigned_abs()).unwrap_or(u32::MAX);
        if remaining > threshold {
            remaining -= threshold + 1;
            drops += 1;
        } else {
            return drops;
        }
    }
}

/// The effective value of `characteristic` after aging: the bought score lowered
/// by the DERIVED aging drops ([`aging_drops`]) and floored at the rules effective
/// minimum (−5), with any free [`Effect::CharacteristicScoreDelta`] bonus (Giant
/// Blood +1 Str/Sta, Dwarf −1) then added on top — so an aged Giant-Blood score
/// can still reach ±6. The aging drop lowers the *bought* score (its threshold is
/// the bought score); the free delta is a separate additive layer. This is what
/// DERIVED / play stats consume; it is deliberately **not** what creation-legality
/// reads (the point-buy budget check in `validation.rs` reads the un-aged bought
/// score from `entity.characteristics`), so entering an already-aged character
/// cannot retroactively make its point-buy illegal.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16579.
pub fn effective_characteristic_after_aging(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let bought = entity
        .characteristics
        .get(&characteristic)
        .copied()
        .map_or(0, i32::from);
    let drops = i32::try_from(aging_drops(entity, characteristic)).unwrap_or(i32::MAX);
    let floor = ruleset
        .characteristic_rules()
        .and_then(|r| r.effective_min_score())
        .map_or(i32::MIN, i32::from);
    let aged = bought.saturating_sub(drops).max(floor);
    aged + characteristic_score_bonus(entity, ruleset, characteristic)
}

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
/// authorizing starting Reputations. Source: Core Rules.md:2512-2514.
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
/// Source: Core Rules.md:2874.
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
/// age band table (Core Rules.md:2366-2374). Data, not hardcoded: the bands live
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
        // instance (Core:4315-4317), unlike a param-chosen single-target Affinity.
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
        // Mastered Spells grants 50 mastery XP (stackable, Core:4471-4474);
        // Flawless Magic floors every spell's mastery at 1 (Core:3887-3889).
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
        // (Core:9518, :15952). With no mastery Virtue it draws the general pool.
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
        // Mastered Spells' +50 pool (Core:4471-4474) is spendable only on Spell
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
        // Spell-Mastery Advancement Totals, halving the XP charged. Core:3887-3889.
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
        // Brother-Knight + Temporal Influence; Core:5113-5116) folds the granted
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
        // (Core:4347-4349); Redcap 50 (Core:4842-4846).
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
        // Demonic Blood confers Infernal Might (Corpus) 5 (RoP:Infernal:4120) and
        // up to 30 levels of Infernal Powers (RoP:Infernal:4122). Effective Might =
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
        // Demonic Might: Infernal Might +2 (RoP:Infernal:4136). Demonic Powers:
        // +20 power levels (RoP:Infernal:4142). Both stack on Demonic Blood.
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
        // True Faith grants a derived True Faith Score of 1 (Core:5169-5171),
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
        // from the grant's declared score. Core:7019-7021, :16464-16475.
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
    /// Magic's 5 = 15 points → Warping Score 2 (Core:16464-16475: cumulative
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

    // --- Issue E: warping-owed V/F (Core:16547-16561) ------------------------

    /// The owed-V/F threshold curve, tested on the pure `from_score` at the rule's
    /// boundary scores so the assertion is independent of the advancement table:
    /// 0 → none; 1 → 1 Minor Flaw; 3 → 2 Minor Flaws; 5 → +supernatural Minor
    /// Virtue; 6 → +1 Major Flaw; 7 → 2 Major Flaws. Source: Core:16553-16561.
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
    /// gives magi Wizard's Twilight instead (Core:16551).
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
    /// (15 ≤ 17 < 30). Core:16617.
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
        assert_eq!(aging_drops(&e, Characteristic::Str), 2);
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
        // Core Rules.md:16613: a Communication of +2 drops to +1 in the year it
        // gains its THIRD aging point; a Stamina of −3 drops to −4 on its FOURTH.
        let rs = xp_ruleset();
        let mut com = xp_entity(vec![]);
        com.characteristics.insert(Characteristic::Com, 2);
        com.aging_points.insert(Characteristic::Com, 2); // ≤ |2|, no drop yet
        assert_eq!(aging_drops(&com, Characteristic::Com), 0);
        assert_eq!(
            effective_characteristic_after_aging(&com, &rs, Characteristic::Com),
            2
        );
        com.aging_points.insert(Characteristic::Com, 3); // the third point drops it
        assert_eq!(aging_drops(&com, Characteristic::Com), 1);
        assert_eq!(
            effective_characteristic_after_aging(&com, &rs, Characteristic::Com),
            1
        );

        let mut sta = xp_entity(vec![]);
        sta.characteristics.insert(Characteristic::Sta, -3);
        sta.aging_points.insert(Characteristic::Sta, 3); // ≤ |−3|, no drop yet
        assert_eq!(aging_drops(&sta, Characteristic::Sta), 0);
        sta.aging_points.insert(Characteristic::Sta, 4); // the fourth point drops it
        assert_eq!(aging_drops(&sta, Characteristic::Sta), 1);
        assert_eq!(
            effective_characteristic_after_aging(&sta, &rs, Characteristic::Sta),
            -4
        );
    }

    #[test]
    fn aging_drop_applies_to_bought_score_then_free_delta_stacks_on_top() {
        // Decision: the aging drop lowers the *bought* score (its threshold uses
        // the bought score per Core Rules.md:16579/:16613); the free
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
        assert_eq!(aging_drops(&e, Characteristic::Str), 1);
        assert_eq!(
            effective_characteristic_after_aging(&e, &rs, Characteristic::Str),
            5
        );
    }

    #[test]
    fn effective_summary_maps_report_aged_value_and_drops() {
        // Core Rules.md:16613 worked example: Communication +2 with 3 aging points
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

        let drops = characteristic_aging_drops(&e);
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
        assert!(characteristic_aging_drops(&e).is_empty());
    }

    #[test]
    fn size_delta_sums_from_virtues_and_flaws() {
        // Size is a derived stat (base 0) modified by SizeDelta effects
        // (Giant Blood +2, Dwarf -2). Core:3975-3978, :5996-5998.
        let rs = xp_ruleset();
        assert_eq!(size(&xp_entity(vec![]), &rs), 0);
        assert_eq!(size(&xp_entity(vec![sel("virtue.giant_blood")]), &rs), 2);
        assert_eq!(size(&xp_entity(vec![sel("flaw.dwarf")]), &rs), -2);
    }

    #[test]
    fn giant_blood_grants_free_characteristic_bonus_reaching_six() {
        // Giant Blood adds a free +1 to Str and Sta that may raise the effective
        // score as high as +6 (Core:3975-3978). The bought score is untouched.
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
        // Giant Blood grants a free +1 to Str and +1 to Sta (Core:3975-3978). The
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
        // Weak Characteristics removes 3 budget points (Core:7056-7058); the grant
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
    /// redistribution hand-computed from the worked example (Core:3731-3737).
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
    /// Te + Fo + Int + Magic Theory + 3 (Core:2465), one entry per Te×Fo combo.
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
    /// budget), so the spell-levels bar can show "base + bonus" the way the XP bar
    /// lists its extra pools beside the general one.
    #[test]
    fn spell_levels_bonus_is_reported_separately_from_the_base() {
        let rs = ruleset();
        // No spell-levels V/F → no bonus to report.
        assert_eq!(spell_levels_bonus(&entity(vec![]), &rs), 0);
        // Skilled Parens contributes its +30 as a standalone signed figure.
        let e = entity(vec![Selection::new(Id::new("virtue.skilled_parens"))]);
        assert_eq!(spell_levels_bonus(&e, &rs), 30);
    }
}
