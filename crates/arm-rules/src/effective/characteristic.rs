//! Characteristic scoring: buy caps/floors (Great/Poor Characteristic), free
//! score deltas (Characteristic Score Delta), Size, and the aging-drop readout.
//! Split out of `effective.rs` (Viktor's V4 architecture finding — 93 free
//! functions across 7 unrelated domains in one file); pure code motion, no
//! behavior change.

use super::*;

/// A non-zero free effective-score bonus targeting one Characteristic (Giant
/// Blood +1 Str/Sta, Dwarf -1). Serializes for the frontend as
/// `{ "characteristic": "str", "bonus": N }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacteristicBonus {
    /// The affected Characteristic.
    pub characteristic: Characteristic,
    /// The summed free bonus (may be negative).
    pub bonus: i32,
}

/// The highest score `characteristic` may be **bought** to: the ruleset's base
/// cap, which is the top row of the printed point-buy table (+3).
///
/// Nothing widens it. Great (Characteristic) does not unlock a purchase — it
/// *performs the raise itself* ("You may raise any Characteristic … by one
/// point"), which the engine models as a free
/// [`Effect::CharacteristicScoreDeltaParam`] read by
/// [`characteristic_score_bonus`]. The printed table stops at ±3 and prices no
/// +4, so there is no cost a cap-shift reading could charge.
///
/// Source: ArMDE:2346-2354 (the table),
/// :4105 (the +3 cap on the bought score), :3987-3989 (Great grants the point).
pub fn characteristic_cap(ruleset: &Ruleset, _characteristic: Characteristic) -> i32 {
    ruleset
        .characteristic_rules()
        .and_then(|rules| rules.base_max_score())
        .map_or(0, i32::from)
}

/// The lowest score `characteristic` may be **bought** to: the ruleset's base
/// floor (-3), the bottom row of the printed table. The sign-mirror of
/// [`characteristic_cap`] — Poor (Characteristic) lowers the score itself rather
/// than opening headroom to sell into.
///
/// Source: ArMDE:2346-2354 (the table),
/// :6598-6600 (Poor lowers the score).
pub fn characteristic_floor(ruleset: &Ruleset, _characteristic: Characteristic) -> i32 {
    ruleset
        .characteristic_rules()
        .and_then(|rules| rules.base_min_score())
        .map_or(0, i32::from)
}

/// The per-characteristic buy cap for all eight characteristics, keyed by
/// characteristic — the spinner ceiling the UI enforces. Every characteristic
/// has a cap, so none is omitted. The map stays per-characteristic because that
/// is the frontend's contract (`effective_dto.rs::EffectiveScores`) and the shape a
/// future book's per-Characteristic buy limit would need; today every entry is
/// the same base cap.
pub fn characteristic_caps(ruleset: &Ruleset) -> BTreeMap<Characteristic, i32> {
    Characteristic::ALL
        .into_iter()
        .map(|c| (c, characteristic_cap(ruleset, c)))
        .collect()
}

/// The per-characteristic buy floor for all eight characteristics, keyed by
/// characteristic — the spinner floor the UI enforces. Every characteristic has
/// a floor, so none is omitted.
pub fn characteristic_floors(ruleset: &Ruleset) -> BTreeMap<Characteristic, i32> {
    Characteristic::ALL
        .into_iter()
        .map(|c| (c, characteristic_floor(ruleset, c)))
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
/// (Large +1, Giant Blood +2, Small Frame -1, Dwarf -2), summed across
/// selections. Size is not a bought Characteristic — it has no cost and no buy
/// cap. Source: ArMDE:3975-3978,
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

/// The clamped contribution of a [`CharacteristicDeltaCap::WithinBase`]
/// [`Effect::CharacteristicScoreDeltaParam`] (B4/Q-51) — `amount` reduced so
/// `bought + contribution` never crosses the ruleset's base cap (positive
/// `amount`) or floor (negative `amount`). Keyed on `cap`, never on whether
/// the effect happens to carry a `gate` — the two are independent axes
/// (coordinator review, post-B4). Falls back to the raw `amount`, unclamped,
/// when the ruleset carries no `characteristic_rules` at all (lean test
/// fixtures) — permissive by default, on
/// `validate_characteristic_delta_preconditions`'s own
/// `let Some(rules) = ... else { return }` precedent.
fn capped_characteristic_delta_contribution(
    ruleset: &Ruleset,
    entity: &Entity,
    characteristic: Characteristic,
    amount: i8,
) -> i32 {
    let Some(rules) = ruleset.characteristic_rules() else {
        return i32::from(amount);
    };
    let bought = i32::from(
        entity
            .characteristics
            .get(&characteristic)
            .copied()
            .unwrap_or(0),
    );
    match amount.signum() {
        1 => match rules.base_max_score() {
            Some(cap) => (i32::from(cap) - bought).clamp(0, i32::from(amount)),
            None => i32::from(amount),
        },
        -1 => match rules.base_min_score() {
            Some(floor) => (i32::from(floor) - bought).clamp(i32::from(amount), 0),
            None => i32::from(amount),
        },
        _ => 0,
    }
}

/// The free effective-score bonus a virtue/flaw grants to `characteristic`,
/// summed across selections. Costs no buy points and stacks on top of the bought
/// score. Two shapes contribute, and both are free:
///
/// - [`Effect::CharacteristicScoreDelta`] names a *fixed* Characteristic — Giant
///   Blood +1 Str/Sta (`ArMDE:3977`), Dwarf -1 (`ArMDE:5996-5998`).
/// - [`Effect::CharacteristicScoreDeltaParam`] names the one the selection
///   targets — Great (Characteristic) +1 (`ArMDE:3989`), Poor -1
///   (`ArMDE:6600`).
///
/// **No ceiling is applied to an `AboveBase` delta, deliberately.**
/// `ArMDE:3977` says Giant Blood's bonus "may raise your scores in those
/// Characteristics as high as +6", so a clamp at +5 would be wrong; Great's
/// own "to no more than +5" falls out of its `max_per_target: 2` over a
/// bought score capped at +3. This is [`CharacteristicDeltaCap::AboveBase`]
/// (the default) — data, never inferred from whether the effect happens to
/// carry a `gate` (coordinator review, post-B4).
///
/// **A `WithinBase` delta is clamped to the base cap/floor instead.** Magical
/// Blood's Magic Human clause reads "may increase one of his Characteristics
/// by 1, but not above +3" (ArMDE:4367) — the OPPOSITE shape from Great
/// Characteristic: no "must already be at the cap" precondition
/// (`validate_characteristic_delta_preconditions` skips `WithinBase` deltas
/// entirely), but the contribution itself must never push the bought score
/// past the printed cap/floor. `(base_max - bought).clamp(0, amount)` for a
/// positive `amount` — zero once bought is already at the cap, `amount` in
/// full while there is still room — sign-mirrored for a negative `amount`
/// against `base_min`, on the same precedent even though no shipped entry
/// uses that direction yet.
pub fn characteristic_score_bonus(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> i32 {
    let mut bonus = 0;
    for_each_effect!(entity, ruleset, |selection, effect| {
        // Exhaustive match so adding an Effect variant is a compile error here,
        // not a silently-ignored bonus. Both interesting arms are guarded, which
        // is what keeps the shared tail sound.
        match effect {
            Effect::CharacteristicScoreDelta {
                characteristic: target,
                amount,
            } if Characteristic::from_id(target) == Some(characteristic) => {
                bonus += i32::from(*amount);
            }
            Effect::CharacteristicScoreDeltaParam {
                param,
                amount,
                gate,
                cap,
            } if selection
                .params
                .get(param)
                .and_then(SelectionParamValue::as_single)
                .and_then(Characteristic::from_id)
                == Some(characteristic) =>
            {
                // Coordinator review, post-B4: `gate` governs WHETHER this
                // delta applies at all (an ungated entry always does); `cap`
                // — data, never inferred from `gate.is_some()` — governs HOW
                // the contribution is computed once it does.
                let active = gate.as_ref().is_none_or(|g| g.holds(selection));
                if active {
                    bonus += match cap {
                        CharacteristicDeltaCap::AboveBase => i32::from(*amount),
                        CharacteristicDeltaCap::WithinBase => {
                            capped_characteristic_delta_contribution(
                                ruleset,
                                entity,
                                characteristic,
                                *amount,
                            )
                        }
                    };
                }
            }
            // Targets another Characteristic, or is not a score delta at all.
            // CharacteristicPoints grants budget, not a score, and is read by
            // characteristic_points_granted.
            irrelevant_effect_variants!() => {}
        }
    });
    bonus
}

/// The effective score of `characteristic`: the bought score (±3) plus any free
/// delta from [`characteristic_score_bonus`]. Great (Characteristic) taken twice
/// reaches +5; Giant Blood stacked on top reaches the +6 `ArMDE:3977` allows.
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
/// a character whose Virtues exempt him from Characteristic aging (`ArMDE:5189`).
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
