//! Characteristic scoring: buy caps/floors (Great/Poor Characteristic), free
//! score deltas (Characteristic Score Delta), Size, and the aging-drop readout.
//! Split out of `effective.rs` (Viktor's V4 architecture finding — 93 free
//! functions across 7 unrelated domains in one file); pure code motion, no
//! behavior change.

use super::*;

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
    for_each_effect!(entity, ruleset, |selection, effect| {
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
            irrelevant_effect_variants!() => {}
        }
    });
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

/// Net Characteristic-buy points granted by [`Effect::CharacteristicPoints`],
/// summed across selections. Signed: Improved Characteristics adds +3 each, Weak
/// Characteristics subtracts 3 each; both stack, so the net may be negative.
pub fn characteristic_points_granted(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::CharacteristicPoints { amount } = effect {
            total += i32::from(*amount);
        }
    });
    total
}

/// The character's derived Size: base 0 plus every [`Effect::SizeDelta`]
/// (Large +1, Giant Blood +2, Small Frame −1, Dwarf −2), summed across
/// selections. Size is not a bought Characteristic — it has no cost and no buy
/// cap. Source: Ars Magica - Definitive Edition (Core Rules).md:3975-3978,
/// :4229-4231, :5996-5998, :6767-6769.
pub fn size(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::SizeDelta { amount } = effect {
            total += i32::from(*amount);
        }
    });
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
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::CharacteristicScoreDelta {
            characteristic: target,
            amount,
        } = effect
            && Characteristic::from_id(target) == Some(characteristic)
        {
            bonus += i32::from(*amount);
        }
    });
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
/// entries (canonical order), for the effective-score tooltip breakdown. Empty for
/// a character whose Virtues exempt him from Characteristic aging (`:5189`).
pub fn characteristic_aging_drops(
    entity: &Entity,
    ruleset: &Ruleset,
) -> BTreeMap<Characteristic, u32> {
    Characteristic::ALL
        .into_iter()
        .filter_map(|c| {
            let drops = aging_drops(entity, ruleset, c);
            (drops != 0).then_some((c, drops))
        })
        .collect()
}
