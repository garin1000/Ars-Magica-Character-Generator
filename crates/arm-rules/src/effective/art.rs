//! Art scoring: flat bonuses (Puissant Art) and the Elemental Magic XP-space
//! redistribution. Split out of `effective.rs` (Viktor's V4 architecture finding
//! — 93 free functions across 7 unrelated domains in one file); pure code
//! motion, no behavior change.

use super::*;

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
/// Source: ArMDE:4818-4820 (Puissant
/// Art, +3; may be taken twice, for two different Arts).
pub fn art_bonus(entity: &Entity, ruleset: &Ruleset, art: &Id) -> i32 {
    let mut bonus = 0;
    for_each_effect!(entity, ruleset, |selection, effect| {
        // Exhaustive match so adding an Effect variant is a compile error
        // here, not a silently-ignored bonus.
        match effect {
            Effect::ArtBonus { param, amount }
                if selection
                    .params
                    .get(param)
                    .and_then(SelectionParamValue::as_single)
                    == Some(art) =>
            {
                bonus += i32::from(*amount);
            }
            // Not an art bonus for this target; contributes nothing here.
            irrelevant_effect_variants!() => {}
        }
    });
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
///
/// `pub(crate)`: `effective/spell.rs::spell_level_cap` (X11b) reuses this same
/// set to except a Form requisite from the cap's fold when both it and the
/// spell's primary Form are elemental (ArMDE:3737) — the same exception
/// `derived/casting.rs::fold_requisite` applies to the Casting Total, via its
/// own independent fold (`derived.rs::InPlayMods::elemental_forms`, D4's
/// in-play-conditions territory). The two folds are allowed to walk the same
/// selections twice — same precedent as `lab_total_mod` vs.
/// `in_play_lab_total_mod`.
pub(crate) fn elemental_magic_forms(entity: &Entity, ruleset: &Ruleset) -> Option<BTreeSet<Id>> {
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::ElementalMagic { forms } = effect {
            return Some(forms.clone());
        }
    });
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
/// Source: ArMDE:3731-3737 (21 XP → 11
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
        // Half, rounded up (ArMDE:3731 worked example: 21 → 11).
        bonus_xp += other_xp.div_ceil(2);
    }
    let boosted = table.score_for_xp(own_xp + bonus_xp);
    i32::from(boosted) - i32::from(own_score)
}

/// Every Art the entity holds a Deficiency in — the Art named by each
/// [`Effect::DeficientArt`] selection's parameter.
///
/// A Deficiency halves every total the Art is added to (`ArMDE:5911` Deficient
/// Form, `ArMDE:5915` Deficient Technique), which reaches **two** layers: the
/// in-play Lab/Casting totals (`derived.rs::in_play_mods`, which seeds
/// `InPlayMods::deficient_arts` from here) and the creation-time per-spell level
/// cap (`effective/spell.rs::spell_level_cap`), since `ArMDE:2465` declares that
/// cap to be a Lab Total. This is the one fold both read, so the two can never
/// disagree about which Arts are deficient.
///
/// The `deficient_art` arm is **guarded** on the parameter resolving, which is
/// what makes reusing [`irrelevant_effect_variants`] sound here: the unfilled-
/// parameter case names no Art and falls through with every other variant,
/// exactly as the `if let Some` it replaces did.
pub(crate) fn deficient_arts(entity: &Entity, ruleset: &Ruleset) -> BTreeSet<Id> {
    let mut arts = BTreeSet::new();
    for_each_effect!(entity, ruleset, |selection, effect| {
        match effect {
            Effect::DeficientArt { param } if selection.params.contains_key(param) => {
                arts.extend(
                    selection
                        .params
                        .get(param)
                        .and_then(SelectionParamValue::as_single)
                        .cloned(),
                );
            }
            // Not a resolved Deficiency; names no Art.
            irrelevant_effect_variants!() => {}
        }
    });
    arts
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
