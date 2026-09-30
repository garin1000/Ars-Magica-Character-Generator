//! Read-only **play-stat** totals derived from a finished character (M5 slice 5i).
//!
//! This module is the play-time counterpart to [`crate::effective`]: where
//! `effective.rs` computes *creation-legality* numbers (bought scores + bonuses,
//! caps, XP), `derived.rs` computes the *in-play* totals a character sheet prints —
//! casting, lab, penetration, magic resistance, combat, soak, encumbrance,
//! fatigue, wounds, longevity, decrepitude, and warping.
//!
//! # Invariants
//!
//! - **Pure and read-only.** `(&Entity, &Ruleset) → owned structs`; nothing is
//!   mutated and nothing is persisted. Calling [`derived_totals`] twice on the same
//!   inputs yields byte-identical results (asserted by a purity test).
//! - **Reuses `effective.rs`.** Effective Art / Ability scores come from
//!   `effective.rs`; play stats use the **aging-adjusted** Characteristic
//!   ([`crate::effective::effective_characteristic_after_aging`]), never the
//!   creation-legality bought score. Decrepitude and Warping scores are the
//!   `effective.rs` functions, not reimplemented here.
//! - **No English in the engine.** Every result struct carries **labelled addend
//!   breakdowns** whose `label` is a stable slug id; the UI maps each through a
//!   Fluent `derived-*` key. The engine never emits user-facing prose.
//! - **Consumption, not just exhaustiveness.** The single [`in_play_mods`] fold is
//!   an exhaustive `match` over every [`Effect`] variant, so a new variant is a
//!   compile error here. But exhaustiveness alone does not prove an effect changes
//!   a number — the per-area functions below *read* each folded modifier, and the
//!   unit tests assert the number moves. Those tests are what guarantee the 5b
//!   in-play effects are actually consumed.
//!
//! Surfaced-only 5b families (study / non-standard-casting / wound-recovery) are
//! **listed** as labelled [`SurfacedModifier`]s rather than folded into a
//! simulated number, because the app does not simulate those subsystems.
//!
//! The **aging** family is listed here too, but it is no longer surfaced-only:
//! M6/6b6's `aging.rs` consumes it (`aging_roll` and `longevity_bonus` move the
//! aging total, `living_conditions` the modifier it subtracts, and the two
//! immunity tags gate the drop and the apparent age). It stays in the read-out
//! because the read-out is the character's standing modifier list, and it is the
//! aging step — not this module — that computes the number.
//!
//! Source line ranges (all `ArMDE`) are
//! cited at each computing function and in `crates/arm-rules/RULES.md` (§5i).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::art::ArtType;
use crate::characteristics::Characteristic;
use crate::effective::{
    decrepitude_score, deficient_arts, effective_ability_score, effective_art_score,
    effective_characteristic_after_aging, resolved_spell_level, selections_for_effects,
    warping_points_total, warping_score,
};
use crate::ruleset::{
    ID_ARTES_LIBERALES, ID_CORPUS, ID_CREO, ID_MAGIC_THEORY, ID_PARMA_MAGICA, ID_PENETRATION,
    ID_PHILOSOPHIAE, Ruleset,
};
use crate::types::{
    AdvancementFactor, CastingScope, CombatStat, Effect, Entity, Familiar, HalvableTotal,
    HealthTrack, Id, LoadoutState, LongevitySource, MAX_CORD_SCORE, MagicResistanceEffect,
    SelectionParamValue, SpecialCasting,
};

// --- Non-standard-casting penalty constants (ArMDE:9243-9245) -----------

/// Casting-Score penalty for casting with **no voice** at all (the "None" Words
/// row). Source: ArMDE:9245.
const NO_VOICE_PENALTY: i32 = -10;
/// Casting-Score penalty for casting with **no gestures** at all (the "None"
/// Gestures row). Source: ArMDE:9245.
const NO_GESTURE_PENALTY: i32 = -5;
/// The no-voice-penalty reduction one casting of Quiet Magic grants (soft voice →
/// no penalty, no voice → −5, i.e. +5; a second casting eliminates it). Source:
/// ArMDE:4822-4826.
const QUIET_MAGIC_VOICE_REDUCTION: i32 = 5;
/// The no-gesture-penalty reduction Subtle Magic grants (no gestures → no
/// penalty, i.e. +5). Source: ArMDE:5073-5076.
const SUBTLE_MAGIC_GESTURE_REDUCTION: i32 = 5;

/// A single labelled term in a breakdown. `label` is a **stable slug id**
/// (`"technique"`, `"stamina"`, `"aura"`, …), mapped to a display string through a
/// Fluent `derived-addend-<label>` key on the UI side — never rendered raw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Addend {
    /// Stable slug id, e.g. `"technique"`.
    pub label: String,
    /// The signed contribution to the total.
    pub value: i32,
}

impl Addend {
    fn new(label: &str, value: i32) -> Self {
        Self {
            label: label.to_string(),
            value,
        }
    }
}

/// Sum of an addend list.
fn sum(addends: &[Addend]) -> i32 {
    saturating_i32_sum(addends.iter().map(|a| a.value))
}

/// Widens every term to `i64`, sums, then narrows back to `i32` by **saturating**
/// at `i32::MIN`/`i32::MAX` rather than wrapping.
///
/// Every casting/lab/penetration total folds in [`Entity::aura`], the only
/// unbounded signed field on an [`Entity`]. It is clamped to
/// `AURA_MODIFIER_MIN..=AURA_MODIFIER_MAX` in two places — `Entity::normalize`
/// (`types.rs::Entity::normalize`) on every save, and
/// `migration.rs::load_entity_migrating` on every load — but `arm-rules` is a library
/// whose `derived_totals` is `pub`, so a caller can still hand these totals an
/// entity that went through neither.
///
/// **In this build an unguarded overflow aborts the process, it does not wrap.**
/// The workspace sets `overflow-checks = true` under `[profile.release]` in the
/// root `Cargo.toml`, deliberately, and
/// `tests/overflow_checks_profile.rs::overflow_checks_are_enabled_in_this_profile`
/// locks that in. So a bare `i32` sum here would panic in release exactly as it
/// does in debug — taking every unsaved edit with it — rather than silently
/// wrapping to a nonsensical total, which is what it would do under Cargo's
/// stock release default of `overflow-checks = false`.
///
/// `i64` cannot overflow summing any realistic number of `i32` terms, so this is
/// exact for every legal input and merely clamps the display value for an illegal
/// one — mirroring the `saturating_add`/`i64`-widening pattern already used by
/// `effective/warping.rs::warping_points_total` and
/// `effective/xp.rs::charged_cost`.
///
/// Note that a *saturated* value is not a safe value to add to: folding anything
/// further onto one must go back through this helper, in both directions. The
/// sign of an addend decides which end it overflows, never whether it can.
fn saturating_i32_sum(terms: impl IntoIterator<Item = i32>) -> i32 {
    let total: i64 = terms.into_iter().map(i64::from).sum();
    i32::try_from(total).unwrap_or(if total > 0 { i32::MAX } else { i32::MIN })
}

/// Integer halving, **rounded down** (Deficient Art / halving flaws halve
/// *totals*).
///
/// No halving rule in the book names a rounding direction, so the rulebook
/// default governs: "In most cases, a rule specifies whether you should round up
/// or down, but if it does not, round down." Source: ArMDE:547. `div_euclid` is
/// exactly floor division for a positive divisor — unlike `/`, which truncates
/// toward zero and so reports a negative total one point in the character's
/// favour. Negative totals are ordinary play (a Dominion aura, a newly
/// gauntleted magus, Deficient Technique), not an edge case.
fn halve(x: i32) -> i32 {
    x.div_euclid(2)
}

// --- In-play effect fold (the single exhaustive-match consumer) ------------

/// Every in-play (5b) modifier folded off the entity's selections + grants, plus
/// the surfaced-only families collected for listing. Built by one exhaustive
/// `match` ([`in_play_mods`]) so a new [`Effect`] variant fails to compile until
/// handled here.
#[derive(Debug, Default, Clone)]
struct InPlayMods {
    /// The magus holds a Magical Focus (a within-focus total is computed).
    has_focus: bool,
    /// The magus holds the Masterpiece Virtue (a lesser-item cap is surfaced).
    has_masterpiece: bool,
    /// Flat Casting-Total modifiers with their scope (Method Caster +3, …).
    casting_mods: Vec<(i32, CastingScope)>,
    /// Flat Lab-Total modifier (Inventive Genius +3, summed).
    lab_mod: i32,
    /// Lab-Total modifier that applies only within a Magical Focus (Potent
    /// Magic's +6/+3, D4) — added to `within_focus` alone, never to `lab_mod`.
    lab_mod_within_focus: i32,
    /// The deficient Technique/Form Art ids (Deficient Art halves totals adding one).
    deficient_arts: BTreeSet<Id>,
    /// Whole-total halvings in effect (Weak Magic → Penetration, Weak Enchanter →
    /// lab enchanting).
    halvings: BTreeSet<HalvableTotal>,
    /// Flat Soak modifier (Tough +3, Frail −1, summed).
    soak_mod: i32,
    /// Flat combat-total modifiers per stat (summed), applying to every weapon.
    combat_mods: BTreeMap<CombatStat, i32>,
    /// Per-weapon *deltas* on top of `combat_mods`, for items that scope a figure
    /// to one weapon (Lame's -3 on Dodge). Each entry is the scoped amount minus
    /// the same item's unscoped amount for that stat, so adding it to the summed
    /// `combat_mods` yields the scoped figure while leaving every *other* item's
    /// unscoped contribution intact. Keyed weapon → stat.
    weapon_combat_mods: BTreeMap<Id, BTreeMap<CombatStat, i32>>,
    /// Health-track penalty deltas per track (positive reduces the penalty).
    health_mods: BTreeMap<HealthTrack, i32>,
    /// Forms whose own contribution to Magic Resistance is dropped (Limited
    /// Magic Resistance, one copy per Form).
    no_form_bonus_forms: BTreeSet<Id>,
    /// Forms against which the Parma contribution to Magic Resistance is halved
    /// (Flawed Parma Magica, one copy per Form).
    halved_parma_forms: BTreeSet<Id>,
    /// Sum of every active [`MagicResistanceEffect::AuraBonus`] (X6a/e1-e2:
    /// Commanding Aura's flat MR bonus, gate-filtered). Read by
    /// `derived/casting.rs::magic_resistance`.
    aura_bonus: i32,
    /// Total no-voice-penalty reduction from Quiet Magic (+5 per casting; a second
    /// casting eliminates the penalty once clamped).
    voice_reduction: i32,
    /// Total no-gesture-penalty reduction from Subtle Magic (+5).
    gesture_reduction: i32,
    /// Forms with Deft Form: casting in them suffers no non-standard voicing or
    /// gesture penalty at all.
    deft_forms: BTreeSet<Id>,
    /// Surfaced-only families, listed rather than folded into a number.
    surfaced: Vec<SurfacedModifier>,
}

/// The `lab_total_mod` carriers D4 (`docs/vf-audit/decisions.md`) resolves as
/// **never** true at character generation, and so hard-excludes from the
/// in-play Lab Total grid (X7a, `tmp/x7a-handover.md`): Adept Laboratory
/// Student and Weak Scholar apply only "when working from the lab texts of
/// others" (ArMDE:3368-3371, :7080-7083), and Cyclic Magic's Virtue half only
/// "if the positive part of the cycle covers the whole season" (ArMDE:3635-3638)
/// — creation fixes no season. `virtue.aristotelian_training` needs no entry
/// here: its `lab_total_mod` effect is deleted at the source instead (X7a item
/// 5), since D4 says its condition can never be satisfied by anything this app
/// models, unlike the other three, which are merely undecided per character.
const D4_EXCLUDED_FROM_LAB_GRID: &[&str] = &[
    "virtue.adept_laboratory_student",
    "flaw.weak_scholar",
    "virtue.cyclic_magic_positive",
];

/// D4's other split: both Potent Magic entries apply "only within the chosen
/// [Magical] focus" (ArMDE:4740-4781), never to the ordinary Lab Total. They
/// are excluded from the flat `in_play_lab_total_mod` fold and instead folded
/// separately by `in_play_lab_total_mod_within_focus`, added to `within_focus`
/// alone (`derived/lab.rs::lab_totals`).
const D4_WITHIN_FOCUS_ONLY: &[&str] = &["virtue.potent_magic_major", "virtue.potent_magic_minor"];

/// `flaw.cyclic_magic_negative`'s cycle-type parameter key, and the one value
/// (of `cycle.solar`/`cycle.lunar`/`cycle.seasonal`) that suppresses its Lab
/// Total penalty in the in-play grid (D52): a seasonal cycle makes the
/// negative half align with season boundaries, reintroducing the same
/// "which season am I in" uncertainty that keeps the Virtue's bonus out —
/// solar/lunar cycles always leave negative time within every season, so the
/// penalty stays certain (and applies) for those.
const CYCLIC_MAGIC_NEGATIVE: &str = "flaw.cyclic_magic_negative";
const CYCLE_PARAM_KEY: &str = "cycle";
const CYCLE_SEASONAL: &str = "cycle.seasonal";

/// D4's per-entry-resolved Lab-Total-modifier fold for the in-play Lab Total
/// grid (`derived/lab.rs::lab_totals`, `creo_corpus_lab_total`, and via those,
/// the familiar and Masterpiece read-outs) — **separate** from
/// `effective::lab_total_mod` (D1), which stays flat and condition-free and is
/// consumed only by `effective/spell.rs::spell_level_cap`. The two folds walk
/// the same selections but disagree on purpose: D1 is a generous ceiling on
/// which spells may be chosen, D4 is a played-out number a character sheet
/// prints.
///
/// Every carrier not named in [`D4_EXCLUDED_FROM_LAB_GRID`] or cycle-gated
/// below applies flat here too — Inventive Genius and Creative Block because
/// their condition ("not using a Lab Text or being taught") is the
/// character-generation default, and both Potent Magic entries because their
/// `within_focus` split is X7b-d's, not this fold's, concern.
///
/// There is no exhaustive-match safeguard here (unlike [`Effect`]'s variants):
/// this is a per-*entry* distinction, not a per-*variant* one. See
/// `lab_total_mod_carriers_match_the_d4_table` for the enumeration test that
/// stands in for one, so a tenth carrier cannot silently default to the wrong
/// fold.
fn in_play_lab_total_mod(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0i32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let item_ref = selection.item_ref.as_str();
        if D4_EXCLUDED_FROM_LAB_GRID.contains(&item_ref) || D4_WITHIN_FOCUS_ONLY.contains(&item_ref)
        {
            continue;
        }
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        let seasonal_cyclic_negative = item_ref == CYCLIC_MAGIC_NEGATIVE
            && selection
                .params
                .get(CYCLE_PARAM_KEY)
                .and_then(SelectionParamValue::as_single)
                == Some(&Id::new(CYCLE_SEASONAL));
        if seasonal_cyclic_negative {
            continue;
        }
        for effect in &item.effects {
            if let Effect::LabTotalMod { amount } = effect {
                total += i32::from(*amount);
            }
        }
    }
    total
}

/// D4's within-focus-only half of the same fold: Potent Magic's flat bonus,
/// summed separately so it never reaches `total`/`base`, only `within_focus`
/// (`derived/lab.rs::lab_totals`).
fn in_play_lab_total_mod_within_focus(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0i32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let item_ref = selection.item_ref.as_str();
        if !D4_WITHIN_FOCUS_ONLY.contains(&item_ref) {
            continue;
        }
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::LabTotalMod { amount } = effect {
                total += i32::from(*amount);
            }
        }
    }
    total
}

/// Folds all in-play (5b) effects off the entity's selections and grants. The
/// `match` is exhaustive: creation-effect and elemental variants are explicit
/// no-ops (consumed by `effective.rs`), so adding an [`Effect`] variant is a
/// compile error until it is classified here.
fn in_play_mods(entity: &Entity, ruleset: &Ruleset) -> InPlayMods {
    // Deficiencies are folded by `effective/art.rs::deficient_arts` rather than in
    // the match below, because the creation-time per-spell level cap needs the same
    // set: `ArMDE:2465` makes that cap a Lab Total, so a Deficiency halves it too.
    // One fold, so the in-play totals and the cap can never disagree about which
    // Arts are deficient. `lab_mod` is folded by `in_play_lab_total_mod` (D4,
    // X7a) — a *different* fold from `effective/spell.rs::lab_total_mod` (D1),
    // which stays flat and condition-free for `spell_level_cap` only; the two
    // are deliberately allowed to disagree per `in_play_lab_total_mod`'s own
    // doc comment.
    let mut m = InPlayMods {
        deficient_arts: deficient_arts(entity, ruleset),
        lab_mod: in_play_lab_total_mod(entity, ruleset),
        lab_mod_within_focus: in_play_lab_total_mod_within_focus(entity, ruleset),
        ..InPlayMods::default()
    };
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        // This item's own unscoped CombatMod figure per stat — the baseline a
        // weapon-scoped figure on the same item replaces. Summed rather than
        // overwritten so a malformed item with two unscoped figures on one stat
        // still yields the amount that was actually folded above.
        let unscoped_combat =
            item.effects
                .iter()
                .fold(BTreeMap::<CombatStat, i32>::new(), |mut acc, effect| {
                    if let Effect::CombatMod {
                        amount,
                        target,
                        weapon: None,
                    } = effect
                    {
                        *acc.entry(*target).or_default() += i32::from(*amount);
                    }
                    acc
                });
        for effect in &item.effects {
            match effect {
                Effect::MagicalFocus { .. } => m.has_focus = true,
                Effect::MasterpieceItem => m.has_masterpiece = true,
                Effect::CastingTotalMod { amount, scope } => {
                    m.casting_mods.push((i32::from(*amount), *scope));
                }
                // Already folded, above — listed so the match stays exhaustive.
                Effect::LabTotalMod { .. } => {}
                // Consumed only by `effective/spell.rs::spell_level_cap` (D28); a
                // no-op for every in-play total this module computes.
                Effect::HalvesSpellCapBeyondTouch => {}
                Effect::DeficientArt { .. } => {}
                // A creation-time training marker, not an in-play total;
                // no-op here — Casting/Lab Totals themselves are unaffected by
                // *how* training was acquired.
                Effect::ConfersHermeticTraining => {}
                // The conditional sibling (D3/R3-1): identical reasoning.
                Effect::ConfersHermeticTrainingIf { .. } => {}
                Effect::MagicTotalHalving { total } => {
                    m.halvings.insert(*total);
                }
                // X6a/e1: an inactive gate contributes nothing — the same
                // idiom `AbilityRef::active_for` already follows.
                Effect::SoakMod { amount, gate } => {
                    if gate.as_ref().is_none_or(|g| g.holds(selection)) {
                        m.soak_mod += i32::from(*amount);
                    }
                }
                // A weapon-scoped figure replaces this item's unscoped one on that
                // weapon alone, so it is stored as the delta between them: adding
                // it to the summed unscoped total yields the scoped figure while
                // every *other* item's unscoped contribution survives untouched.
                Effect::CombatMod {
                    amount,
                    target,
                    weapon: None,
                } => {
                    *m.combat_mods.entry(*target).or_default() += i32::from(*amount);
                }
                Effect::CombatMod {
                    amount,
                    target,
                    weapon: Some(weapon),
                } => {
                    let replaced = unscoped_combat.get(target).copied().unwrap_or(0);
                    *m.weapon_combat_mods
                        .entry(weapon.clone())
                        .or_default()
                        .entry(*target)
                        .or_default() += i32::from(*amount) - replaced;
                }
                Effect::HealthMod { track, amount } => {
                    *m.health_mods.entry(*track).or_default() += i32::from(*amount);
                }
                // Two kinds fold into the flat per-Form MR number
                // (magic_resistance()), and both are scoped to the ONE Form the
                // selection names — the rulebook buys each of them "for different
                // Forms", one copy each, so an unscoped fold would apply a Minor
                // Flaw to all ten Forms at once. The Form travels exactly as Deft
                // Form's does: through the effect's `param` key into the
                // selection's own params. A copy that names no Form applies to
                // none — `missing_param` asks for the choice rather than this
                // guessing one.
                // Source: ArMDE:6346-6349 (Limited Magic Resistance, "one of your
                // Form scores"), :6142-6145 (Flawed Parma, "against a certain
                // Form").
                //
                // The realm-conditional / situational variants (aura bonus, realm
                // susceptibilities, the conditional Penetration waiver) cannot be
                // folded into that flat figure at all, so they are surfaced
                // labelled rather than silently dropped.
                // Source: ArMDE:6819-6826
                // (the two realm susceptibilities that do halve MR), :3579-3596
                // (Commanding Aura) & :4998-5001 (Special Circumstances) for
                // aura_bonus, :7068-7070 (Weak Magic Resistance).
                Effect::MagicResistanceMod {
                    kind,
                    param,
                    amount,
                    gate,
                } => match kind {
                    MagicResistanceEffect::NoFormBonus => {
                        if let Some(form) = param
                            .as_ref()
                            .and_then(|p| selection.params.get(p))
                            .and_then(SelectionParamValue::as_single)
                        {
                            m.no_form_bonus_forms.insert(form.clone());
                        }
                    }
                    MagicResistanceEffect::HalvedParma => {
                        if let Some(form) = param
                            .as_ref()
                            .and_then(|p| selection.params.get(p))
                            .and_then(SelectionParamValue::as_single)
                        {
                            m.halved_parma_forms.insert(form.clone());
                        }
                    }
                    // X6a/e1-e2: Commanding Aura's flat bonus — folded into a
                    // number `magic_resistance()` reads, unlike the three
                    // scene-conditional kinds below, which stay surfaced-only.
                    // An inactive gate contributes nothing.
                    MagicResistanceEffect::AuraBonus => {
                        if gate.as_ref().is_none_or(|g| g.holds(selection)) {
                            m.aura_bonus += *amount;
                        }
                    }
                    MagicResistanceEffect::SusceptibleFaerie
                    | MagicResistanceEffect::SusceptibleInfernal
                    | MagicResistanceEffect::ConditionalPenetrationWaiver => {
                        m.surfaced.push(SurfacedModifier {
                            family: ModifierFamily::MagicResistance,
                            detail: kind.to_string(),
                            amount: 0,
                            factor: None,
                            source: Some(item.id.clone()),
                            ability: None,
                        })
                    }
                },
                // Surfaced-only families: listed labelled, never simulated. Aging is
                // the exception — `aging.rs` consumes it (M6/6b6); it is listed here
                // as a standing modifier, not because nothing reads it.
                Effect::AgingMod { kind, amount } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::Aging,
                    detail: kind.to_string(),
                    amount: i32::from(*amount),
                    factor: None,
                    source: Some(item.id.clone()),
                    ability: None,
                }),
                // D55: `amount` and `factor` are mutually exclusive and load-time
                // validated (`ruleset/integrity.rs::validate_advancement_mod_shape`),
                // so exactly one is `Some` here. `unwrap_or(0)` on the amount side is
                // therefore never a silent "no magnitude" collision with a real
                // factor row: `factor` is what a reader (and the UI) checks first.
                Effect::AdvancementMod {
                    source,
                    amount,
                    factor,
                } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::Advancement,
                    detail: source.to_string(),
                    amount: amount.map(i32::from).unwrap_or(0),
                    factor: *factor,
                    source: Some(item.id.clone()),
                    ability: None,
                }),
                // Non-standard-casting relievers are computed into the per-cell
                // NonStandardCasting variants; every other quirk stays surfaced.
                Effect::SpecialCastingMod { kind, param } => match kind {
                    SpecialCasting::QuietWords => m.voice_reduction += QUIET_MAGIC_VOICE_REDUCTION,
                    SpecialCasting::SubtleGestures => {
                        m.gesture_reduction += SUBTLE_MAGIC_GESTURE_REDUCTION
                    }
                    SpecialCasting::DeftForm => {
                        if let Some(form) = param
                            .as_ref()
                            .and_then(|p| selection.params.get(p))
                            .and_then(SelectionParamValue::as_single)
                        {
                            m.deft_forms.insert(form.clone());
                        }
                    }
                    SpecialCasting::Diedne
                    | SpecialCasting::FaerieRaised
                    | SpecialCasting::LifeLinkedSpontaneous
                    | SpecialCasting::SpellImprovisation
                    | SpecialCasting::Mercurian
                    | SpecialCasting::LifeBoost
                    | SpecialCasting::Circumstantial
                    | SpecialCasting::DoubledAuraPenalty => m.surfaced.push(SurfacedModifier {
                        family: ModifierFamily::SpecialCasting,
                        detail: kind.to_string(),
                        amount: 0,
                        factor: None,
                        source: Some(item.id.clone()),
                        ability: None,
                    }),
                },
                Effect::AbilityRollModParam { param, amount } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::AbilityRoll,
                    detail: selection
                        .params
                        .get(param)
                        .and_then(SelectionParamValue::as_single)
                        .map(|id| id.as_str().to_string())
                        .unwrap_or_default(),
                    amount: i32::from(*amount),
                    factor: None,
                    source: Some(item.id.clone()),
                    // A free-text subject, not a catalogue Ability id — the
                    // structured field is reserved for the fixed-target shape
                    // below (coordinator review, post-B5-phase-1).
                    ability: None,
                }),
                // B5/F-489: the fixed-target twin, named directly by the entry
                // (Poor Hearing: -3 to Awareness). `ability` is structured,
                // never `detail` — the UI resolves it through ruleset i18n
                // (`abilityLabel`'s own path), exactly like a bought Ability's
                // name, so a raw catalogue id never reaches a field the
                // component renders verbatim.
                // X6a/e1: an inactive gate surfaces nothing (Faerie Blood's
                // Dwarf-only Craft bonus).
                Effect::AbilityRollMod {
                    ability,
                    amount,
                    gate,
                } => {
                    if gate.as_ref().is_none_or(|g| g.holds(selection)) {
                        m.surfaced.push(SurfacedModifier {
                            family: ModifierFamily::AbilityRoll,
                            detail: String::new(),
                            amount: i32::from(*amount),
                            factor: None,
                            source: Some(item.id.clone()),
                            ability: Some(ability.clone()),
                        });
                    }
                }
                // D69/X7b-e row 42: Lingering Injury's category-wide penalty,
                // multiplied by 1 + Decrepitude Score (ArMDE:6350-6352). The
                // aggravated (-3) alternative stays text (no aggravation
                // tracking exists on `Entity`).
                Effect::DecrepitudeScaledRollMod { amount } => {
                    let multiplier = 1 + i32::from(decrepitude_score(entity, ruleset));
                    m.surfaced.push(SurfacedModifier {
                        family: ModifierFamily::PhysicalActivity,
                        detail: String::new(),
                        amount: i32::from(*amount) * multiplier,
                        factor: None,
                        source: Some(item.id.clone()),
                        ability: None,
                    });
                }
                // Creation-effect variants (consumed by effective.rs) and the
                // Elemental Magic XP-space marker: no in-play modifier here.
                Effect::AbilityBonus { .. }
                | Effect::CharacteristicScoreDeltaParam { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::GroupAffinityCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::ScaledRestrictedAbilityXp { .. }
                // D40/D2: a creation-time life-stage XP replacement, not an
                // in-play total.
                | Effect::ReplacesLifeStageXp { .. }
                // D3: a creation-time general-XP/life-stage-years grant, not
                // an in-play total either.
                | Effect::TruncatedApprenticeshipXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                // Creation-time floor grant (F-63/C5c), not an in-play total.
                | Effect::AbilityScoreGrantParam { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::AbilityBonusGated { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
                | Effect::GrantsSpellMastery { .. }
                | Effect::GrantsSelection { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::TrueFaithGrant { .. }
                // F-256: the relic's own True Faith Score, not an in-play
                // total for the character.
                | Effect::RelicTrueFaith { .. }
                | Effect::WarpingGrant { .. }
                // D69/X7b-e: the parameterized Warping grant (Raised from the
                // Dead) is a creation-time total, not an in-play total;
                // consumed only by `effective/warping.rs`.
                | Effect::WarpingGrantParam { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                // D69/X7b-e: a creation-time buy-cap shift (Uninspirational),
                // not an in-play total; consumed only by
                // `effective/characteristic.rs::characteristic_cap`.
                | Effect::CharacteristicMax { .. }
                | Effect::GrantsReputation { .. }
                // B3/D23/F-542: a narrative Personality-Trait grant, not an
                // in-play total — consumed only by
                // `ItemPredicate::GrantsPersonalityTrait`'s derivation.
                | Effect::GrantsPersonalityTrait
                // D69/X7b-e: creation-legality constraints (Weak
                // Personality's tightened range, Fickle Nature's trait-pair
                // requirement) — no in-play total; consumed only by
                // `validation/scores.rs`.
                | Effect::PersonalityTraitRange { .. }
                | Effect::RequiresPersonalityTraitPair { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                | Effect::FocusPoints { .. }
                | Effect::ElementalMagic { .. }
                | Effect::ForbidsAbilitySpecialties
                | Effect::ForbidsRitualCasting
                | Effect::WaivesAbilityAgeCap
                // B1/D21: creation-time authorization constraints (a
                // category/id forbid) — not an in-play total, no different
                // from `ForbidsAbilitySpecialties` above.
                | Effect::ForbidsAbilityCategory { .. }
                | Effect::ForbidsAbilityCategoryParam { .. }
                | Effect::ForbidsItemCategory { .. }
                | Effect::ForbidsAbilities { .. }
                // X6a/e6: folded only by `ability_age_cap`, not an in-play total.
                | Effect::AbilityScoreCapOverrideParam { .. }
                | Effect::AbilityScoreCapAllExcept { .. } => {}
            }
        }
    }
    m
}

impl InPlayMods {
    /// Sum of flat Casting-Total mods applying to `cast`.
    fn casting_mod_for(&self, cast: CastType) -> i32 {
        self.casting_mods
            .iter()
            .filter(|(_, scope)| cast.matches(*scope))
            .map(|(amount, _)| *amount)
            .sum()
    }

    /// Whether either Art of a `(technique, form)` pair is a Deficient Art.
    fn deficient(&self, technique: &Id, form: &Id) -> bool {
        self.deficient_arts.contains(technique) || self.deficient_arts.contains(form)
    }

    /// The residual Casting-Score penalty for casting a `form` spell with no voice:
    /// Deft Form waives it entirely, else the −10 base plus Quiet Magic reduction,
    /// clamped so a Virtue can never turn it into a bonus. Source:
    /// ArMDE:9245, :4822-4826, :3645-3648.
    fn residual_voice_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_VOICE_PENALTY + self.voice_reduction).min(0)
    }

    /// The residual Casting-Score penalty for casting a `form` spell with no
    /// gestures: Deft Form waives it, else the −5 base plus Subtle Magic reduction,
    /// clamped at 0. Source: ArMDE:9245,
    /// :5073-5076, :3645-3648.
    fn residual_gesture_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_GESTURE_PENALTY + self.gesture_reduction).min(0)
    }
}

/// The four ways a casting total is derived (ArMDE:9095-9145). One breakdown yields
/// all four; the scope filter and post-divisor differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CastType {
    Formulaic,
    Ritual,
    /// Spontaneous (both fatiguing ÷2 and non-fatiguing ÷5 share the same scope
    /// filter; the divisor is applied after this).
    Spontaneous,
}

impl CastType {
    fn matches(self, scope: CastingScope) -> bool {
        match self {
            CastType::Formulaic => matches!(
                scope,
                CastingScope::All | CastingScope::Formulaic | CastingScope::FormulaicRitual
            ),
            CastType::Ritual => matches!(
                scope,
                CastingScope::All | CastingScope::Ritual | CastingScope::FormulaicRitual
            ),
            CastType::Spontaneous => {
                matches!(scope, CastingScope::All | CastingScope::Spontaneous)
            }
        }
    }
}

// --- Characteristic helpers (aging-adjusted, per the play-stat invariant) --

fn characteristic(entity: &Entity, ruleset: &Ruleset, c: Characteristic) -> i32 {
    effective_characteristic_after_aging(entity, ruleset, c)
}

/// The magus's effective score in a Magic-related Ability (Parma, Penetration,
/// Magic Theory, Artes Liberales, Philosophiae), 0 if unheld.
fn ability(entity: &Entity, ruleset: &Ruleset, id: &str) -> i32 {
    effective_ability_score(entity, ruleset, &Id::new(id), None)
}

/// The magus's effective score in an Art (Creo, Corpus, …), by bare slug — the
/// Art-side counterpart of [`ability`], so both catalogue lookups read the same.
fn art(entity: &Entity, ruleset: &Ruleset, id: &str) -> i32 {
    effective_art_score(entity, ruleset, &Id::new(id))
}

mod casting;

pub use casting::{
    CastingTotal, CastingWithinFocus, MagicResistance, NonStandardCasting, PenetrationLine,
    casting_totals, magic_resistance, penetration,
};

mod combat;
mod focus_power;
mod lab;

pub use focus_power::{FocusPowerLine, focus_power_lines};

pub use combat::{
    CombatLine, EncumbranceTotal, FatigueLevel, FatigueTier, SoakTotal, WoundBand, WoundRange,
    combat_totals, encumbrance, fatigue_levels, soak, wound_ranges,
};
pub use lab::{
    LabTotal, LongevityBonus, LongevityHint, MasterpieceCap, lab_totals, longevity_bonus,
    masterpiece_item_cap,
};

// --- Familiar (bonding read-outs) ------------------------------------------

/// What each cord score from 0 to +5 costs in Lab-Total points.
///
/// "The strength of each of these cords is rated from 0 to +5 … a strength of +1
/// requires 5 points, a score of +2 requires 15 points, a score of +3 requires 30
/// points, a score of +4 requires 50 points, and a score of +5 (the maximum)
/// requires 75 points" (ArMDE:10836).
///
/// A fixed five-entry rule curve, so it is a `const` here rather than ruleset data
/// — the same call as [`LOAD_TABLE`] for Encumbrance. RULES.md is its provenance
/// home.
const CORD_COST_TABLE: [u32; 6] = [0, 5, 15, 30, 50, 75];

/// The rules-legal score of a stored cord value, clamped to the +5 maximum
/// (ArMDE:10836).
///
/// **Every** consumer of a cord score routes through this, so the read-outs cannot
/// disagree. `Familiar`'s cord fields are plain `u8`, and a hand-edited or legacy save
/// can therefore carry any value up to 255; left unclamped, the same entered number
/// would render as one figure on the familiar's cord-cost read-out and a wildly
/// different one in Soak ([`soak`]) and on the Longevity Ritual's Bronze-cord line
/// ([`longevity_bonus`]).
///
/// The maximum itself is stated once, in [`MAX_CORD_SCORE`] beside the `Familiar`
/// type, because [`Familiar::normalize`] clamps the *stored* fields to the same
/// bound; this read-side clamp still matters for a value that reaches a consumer
/// before a normalize pass (a freshly loaded save).
fn cord_score(raw: u8) -> u8 {
    raw.min(MAX_CORD_SCORE)
}

/// The entity's Bronze-cord bonus, or 0 when it has no familiar.
///
/// "**The Bronze Cord:** You can apply your bronze cord score as a bonus to Soak
/// rolls and totals, to healing rolls, to rolls to withstand deprivation …, and to
/// rolls to resist aging."
/// (Source: ArMDE:10844)
///
/// The one entity-level accessor for that score, shared by every total the cord
/// feeds: [`soak`], the aging-resistance note on [`longevity_bonus`], and the
/// crisis-survival read-out. It goes **through** [`cord_score`], so the +5 maximum
/// (`ArMDE:10836`) keeps its single home there and cannot be bypassed by adding a
/// consumer here. [`cord_points_spent`] deliberately stays on [`cord_score`]: it
/// prices all three cords of a `Familiar` and has no bronze-only, entity-level
/// form.
pub(crate) fn bronze_cord_bonus(entity: &Entity) -> i32 {
    entity
        .familiar
        .as_ref()
        .map(|f| i32::from(cord_score(f.cord_bronze)))
        .unwrap_or(0)
}

mod familiar;

pub use familiar::{
    FamiliarBinding, FamiliarReadout, TalismanCapacity, cord_points_spent, familiar_binding_level,
    familiar_invested_power_levels, familiar_readout, talisman_capacity,
};

// --- Surfaced-only modifiers -----------------------------------------------

/// The family a surfaced-only 5b modifier belongs to. A fixed, closed taxonomy —
/// every producer in [`in_play_mods`] tags its row with exactly one of these — so
/// it is an enum, rendered via Fluent, never as a raw slug. (The `detail` within a
/// family stays a `String`: it is an open free-text / per-effect subject.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifierFamily {
    /// Aging-roll modifiers (Unaging and similar).
    Aging,
    /// Advancement / study modifiers (Apt Student and similar).
    Advancement,
    /// Non-standard-casting quirks not folded into a casting cell.
    SpecialCasting,
    /// Ability-roll modifiers scoped to a specific Ability.
    AbilityRoll,
    /// Surfaced health-track rolls (fatigue / casting-fatigue / recovery).
    HealthRoll,
    /// Realm-conditional / situational Magic-Resistance modifiers that cannot be
    /// folded into the flat per-Form MR number (aura bonus, realm susceptibilities).
    MagicResistance,
    /// A roll penalty over an unenumerated category of physical-activity rolls,
    /// scaled by Decrepitude (Lingering Injury and similar) — no existing family
    /// fits a category-wide, Decrepitude-scaled penalty.
    PhysicalActivity,
}

impl std::fmt::Display for ModifierFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ModifierFamily::Aging => "aging",
            ModifierFamily::Advancement => "advancement",
            ModifierFamily::SpecialCasting => "special_casting",
            ModifierFamily::AbilityRoll => "ability_roll",
            ModifierFamily::HealthRoll => "health_roll",
            ModifierFamily::MagicResistance => "magic_resistance",
            ModifierFamily::PhysicalActivity => "physical_activity",
        })
    }
}

/// A surfaced-only 5b modifier the app **lists** rather than simulates (study /
/// aging-roll / non-standard-casting / wound-recovery families).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfacedModifier {
    /// The modifier's family; serializes to its stable slug (`"aging"`,
    /// `"advancement"`, `"special_casting"`, `"ability_roll"`, `"health_roll"`),
    /// mapped through Fluent.
    pub family: ModifierFamily,
    /// The scalar/detail slug within the family (an enum's `Display`, or a free-text
    /// subject for ability-roll modifiers).
    pub detail: String,
    /// The modifier amount (0 when the family is a mode toggle, e.g. Unaging,
    /// or when `factor` is present instead — see `factor`'s own doc comment).
    pub amount: i32,
    /// Set only for an `Advancement` row backed by
    /// [`Effect::AdvancementMod`]'s `factor` (D55): the row is multiplicative,
    /// not additive, and `amount` carries no meaning for it. Every other
    /// family always ships `None` here, so `amount: 0` keeps its original,
    /// unambiguous "no magnitude" reading for them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor: Option<AdvancementFactor>,
    /// The id of the Virtue/Flaw/other item whose selection produced this row
    /// (D45, F-423). Before this field existed, two different carriers of the
    /// same `family`/`detail` pair (two Flaws both granting
    /// `SpecialCasting::Circumstantial`) rendered as the identical
    /// unattributed line with nothing to tell them apart. Rendered through the
    /// label map as the item's localized display name — never the raw id,
    /// exactly like `category-<id>`/`magnitude-<id>`.
    ///
    /// `None` only for [`ModifierFamily::HealthRoll`]: unlike the other five
    /// families, which push one row per producing selection directly inside
    /// [`in_play_mods`], a surfaced health track is read back out of
    /// `InPlayMods::health_mods`, which already sums every contributing
    /// selection's amount into one number before [`surfaced_modifiers`] builds
    /// the row — there is no single item left to name by then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Id>,
    /// The FIXED Ability a [`Effect::AbilityRollMod`] row targets (coordinator
    /// review, post-B5-phase-1): a structured id, resolved by the UI through
    /// the same ruleset-i18n path `abilityLabel` already uses for a bought
    /// Ability's own name — never through `detail`, which stays reserved for
    /// the free-text subject a [`Effect::AbilityRollModParam`] row carries
    /// (Academic Concentration). Rendering `detail` verbatim is only ever
    /// correct for player-typed text; a catalogue id must never reach the
    /// same field, or it renders as a bare slug (CLAUDE.md). `None` for every
    /// other family, and for the parameter-relative `AbilityRoll` row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability: Option<Id>,
}

/// Every surfaced-only modifier the character carries, for the read-out list.
/// Source: ArMDE:3422-3425 (Apt
/// Student), :5187-5190 (Unaging), :3645-3682 (non-standard casting).
pub fn surfaced_modifiers(entity: &Entity, ruleset: &Ruleset) -> Vec<SurfacedModifier> {
    let mut m = in_play_mods(entity, ruleset);
    // Surfaced health tracks (roll / recovery / casting-fatigue) are not folded
    // into fatigue/wounds; list them here.
    for (track, amount) in &m.health_mods {
        match track {
            HealthTrack::FatigueRoll | HealthTrack::CastingFatigue | HealthTrack::Recovery => {
                m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::HealthRoll,
                    detail: track.to_string(),
                    amount: *amount,
                    factor: None,
                    source: None,
                    ability: None,
                });
            }
            HealthTrack::FatiguePenalty | HealthTrack::WoundPenalty => {}
        }
    }
    m.surfaced
}

// --- Aggregator ------------------------------------------------------------

/// The full read-only play-stat read-out for a finished character. Serialized to
/// the frontend by the `derived_totals` Tauri command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedTotals {
    /// Whether the character is Hermetically trained (magic totals are present
    /// only then) — `is_hermetically_trained` (D56/A0), not just a real magus
    /// profile: an entity whose selections confer training (e.g. the Abandoned
    /// Apprentice Flaw) reads true here too.
    pub hermetically_trained: bool,
    /// Per-`(Technique, Form)` Lab Totals (magi only; empty otherwise).
    pub lab_totals: Vec<LabTotal>,
    /// Per-`(Technique, Form)` Casting Totals (magi only; empty otherwise).
    pub casting_totals: Vec<CastingTotal>,
    /// Per-known-spell Penetration lines (magi only; empty otherwise).
    pub penetration: Vec<PenetrationLine>,
    /// Per-Form Magic Resistance (magi only; empty otherwise).
    pub magic_resistance: Vec<MagicResistance>,
    /// The Longevity Ritual bonus (magi only; `None` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longevity: Option<LongevityBonus>,
    /// The Masterpiece lesser-item cap (magi with the Virtue only; `None` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub masterpiece: Option<MasterpieceCap>,
    /// The talisman's enchantment capacity in pawns of Vim vis (magi with a
    /// talisman only; `None` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talisman_capacity: Option<TalismanCapacity>,
    /// The familiar bonding read-out (magi with a familiar only; `None` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiar: Option<FamiliarReadout>,
    /// One line per entered Focus Power — magnitude, Initiative and Fatigue cost
    /// (`ArMDE:3899`, `ArMDE:3901`). Empty for a character with none, and open to every
    /// character type, since Focus Power is a Supernatural Virtue rather than a
    /// Hermetic one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_powers: Vec<FocusPowerLine>,
    /// One or two combat lines per equipped weapon — with-shield then bare when a
    /// shield is equipped and the weapon is one-handed, otherwise a single line.
    pub combat: Vec<CombatLine>,
    /// The Soak total.
    pub soak: SoakTotal,
    /// The Encumbrance read-out.
    pub encumbrance: EncumbranceTotal,
    /// The Fatigue levels and penalties.
    pub fatigue: Vec<FatigueLevel>,
    /// The Size-indexed wound ranges.
    pub wounds: Vec<WoundRange>,
    /// The character's derived Size.
    pub size: i32,
    /// The derived Decrepitude Score (reused from `effective.rs`).
    pub decrepitude_score: u8,
    /// The derived Warping Score (reused from `effective.rs`).
    pub warping_score: u8,
    /// The total Warping Points (stored + granted).
    pub warping_points: u32,
    /// The surfaced-only modifier list.
    pub surfaced_modifiers: Vec<SurfacedModifier>,
}

/// Computes the full play-stat read-out for `entity`. Pure and read-only: reuses
/// `effective.rs` for effective scores, Decrepitude, and Warping, and never
/// mutates or recomputes creation legality. Magic totals are computed only for a
/// Hermetically trained entity (`is_hermetically_trained`, D56/A0 — not just a
/// real magus, but also a test fixture — and, once D3 ships, the Abandoned
/// Apprentice — whose selections confer training).
pub fn derived_totals(entity: &Entity, ruleset: &Ruleset) -> DerivedTotals {
    let trained = crate::effective::is_hermetically_trained(
        entity,
        ruleset,
        ruleset.profile(&entity.type_id),
    );
    // A supernatural being (Might Score) has Magic Resistance too, even though it
    // is not a magus. Source: RoP:M:1472.
    let has_might = crate::effective::effective_might(entity, ruleset).is_some();
    // The 5x10 Lab-Total grid, built ONCE. The Masterpiece cap and the familiar
    // binding are both "the best cell of this grid", and each used to rebuild it:
    // three builds per recompute, each cell calling `effective_art_score` twice,
    // each of those cloning the whole selection vector for any character carrying a
    // grant. The store recomputes on every debounced keystroke, so this was the
    // hottest path in the engine. Anything else wanting the grid should take it as
    // a parameter rather than add a fourth build.
    let lab = if trained {
        lab_totals(entity, ruleset)
    } else {
        Vec::new()
    };
    // Bound before the struct literal so both can borrow `lab`, which is then
    // moved into the `lab_totals` field.
    let masterpiece = if trained {
        masterpiece_item_cap(&lab, entity, ruleset)
    } else {
        None
    };
    let familiar = if trained {
        familiar_readout(&lab, entity)
    } else {
        None
    };
    DerivedTotals {
        hermetically_trained: trained,
        lab_totals: lab,
        casting_totals: if trained {
            casting_totals(entity, ruleset)
        } else {
            Vec::new()
        },
        penetration: if trained {
            penetration(entity, ruleset)
        } else {
            Vec::new()
        },
        magic_resistance: if trained || has_might {
            magic_resistance(entity, ruleset)
        } else {
            Vec::new()
        },
        longevity: if trained {
            longevity_bonus(entity, ruleset)
        } else {
            None
        },
        masterpiece,
        talisman_capacity: if trained {
            talisman_capacity(entity, ruleset)
        } else {
            None
        },
        familiar,
        focus_powers: focus_power_lines(entity, ruleset),
        combat: combat_totals(entity, ruleset),
        soak: soak(entity, ruleset),
        encumbrance: encumbrance(entity, ruleset),
        fatigue: fatigue_levels(entity, ruleset),
        wounds: wound_ranges(entity, ruleset),
        size: crate::effective::size(entity, ruleset),
        decrepitude_score: decrepitude_score(entity, ruleset),
        warping_score: warping_score(entity, ruleset),
        warping_points: warping_points_total(entity, ruleset),
        surfaced_modifiers: surfaced_modifiers(entity, ruleset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::RulesetSources;
    use crate::types::{
        AbilityScore, ArtScore, EntityKind, EquipmentSlot, Familiar, FocusPower, LongevityRitual,
        MightScore, Realm, RulesetRef, Selection, SpellSelection, SupernaturalPower, Talisman,
    };
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    /// A magus-capable ruleset: the five combat/magic Abilities, Techniques Creo +
    /// Perdo, Forms Corpus + Ignem + Terram, the Art advancement curve, weapons +
    /// shield + armor, a spell, and the in-play V/F this slice consumes (Method
    /// Caster, a Magical Focus, Deficient Technique, Tough, Weak Magic, Flawed
    /// Parma, Enduring Constitution, Apt Student).
    fn ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.method_caster", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "casting_total_mod", "amount": 3, "scope": "formulaic_ritual" }] },
          { "id": "virtue.magical_focus", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "focus", "type": "ref", "domain": "text" }],
            "effects": [{ "type": "magical_focus", "param": "focus", "major": false }] },
          { "id": "flaw.deficient_technique", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "technique" }],
            "effects": [{ "type": "deficient_art", "param": "art" }] },
          { "id": "flaw.deficient_form", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "deficient_art", "param": "form" }] },
          { "id": "flaw.difficult_longevity_ritual", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "lab_longevity" }] },
          { "id": "flaw.weak_enchanter", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "lab_enchanting" }] },
          { "id": "virtue.tough", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "soak_mod", "amount": 3 }] },
          { "id": "flaw.weak_magic", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "penetration" }] },
          { "id": "flaw.flawed_parma", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "magic_resistance_mod", "kind": "halved_parma", "param": "form" }] },
          { "id": "virtue.enduring_constitution", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [
              { "type": "health_mod", "track": "wound_penalty", "amount": 1 },
              { "type": "health_mod", "track": "fatigue_penalty", "amount": 1 }
            ] },
          { "id": "virtue.apt_student", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "advancement_mod", "source": "taught", "amount": 5 }] },
          { "id": "virtue.good_teacher", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [
              { "type": "advancement_mod", "source": "teaching", "amount": 5 },
              { "type": "advancement_mod", "source": "authoring", "amount": 3 }
            ] },
          { "id": "flaw.incomprehensible", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [
              { "type": "advancement_mod", "source": "teaching", "factor": "half" },
              { "type": "advancement_mod", "source": "authoring", "factor": "half" }
            ] },
          { "id": "flaw.loose_magic", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [
              { "type": "advancement_mod", "source": "spell_mastery", "factor": "half" }
            ] },
          { "id": "virtue.quiet_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "quiet_words" }] },
          { "id": "virtue.subtle_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "subtle_gestures" }] },
          { "id": "virtue.deft_form", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form", "param": "form" }] },
          { "id": "virtue.masterpiece", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "masterpiece_item" }] },
          { "id": "virtue.inventive_genius", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "lab_total_mod", "amount": 3 }] },
          { "id": "flaw.lame", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "combat_mod", "amount": -3, "target": "initiative" }] },
          { "id": "flaw.lame_split", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "combat_mod", "amount": -1, "target": "defense" },
                        { "type": "combat_mod", "amount": -3, "target": "defense",
                          "weapon": "weapon.dodge" }] },
          { "id": "flaw.limited_magic_resistance", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "magic_resistance_mod", "kind": "no_form_bonus", "param": "form" }] },
          { "id": "flaw.susceptibility_to_faerie_power", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_resistance_mod", "kind": "susceptible_faerie" }] },
          { "id": "virtue.true_faith", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "true_faith_grant", "score": 1 }] },
          { "id": "virtue.unaging", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "aging_mod", "kind": "no_aging", "amount": 0 }] },
          { "id": "virtue.diedne_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "diedne" }] },
          { "id": "virtue.life_boost", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "life_boost" }] },
          { "id": "virtue.academic_concentration", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "parameters": [{ "key": "subject", "type": "ref", "domain": "text" }],
            "effects": [{ "type": "ability_roll_mod_param", "param": "subject", "amount": 3 }] },
          { "id": "flaw.weak_spontaneous", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "spontaneous_casting" }] },
          { "id": "virtue.long_winded", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "health_mod", "track": "fatigue_roll", "amount": 3 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.test_confers_training", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "confers_hermetic_training" }] }
        ]"#;
        let types = r#"[
          { "id": "magus", "hermetically_trained": true, "order_member": true,
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "hermetic"], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "gift_policy": "required",
            "gift_categories": [], "creation_phases": ["concept"] },
          { "id": "grog", "hermetically_trained": false, "order_member": false,
            "budget": { "virtue_points": 3, "flaw_points": 3 },
            "permitted_categories": ["general"], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "gift_policy": "forbidden",
            "gift_categories": [], "creation_phases": ["concept"] }
        ]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [
          { "id": "ability.magic_theory", "category": "arcane" },
          { "id": "ability.parma_magica", "category": "arcane" },
          { "id": "ability.penetration", "category": "arcane" },
          { "id": "ability.artes_liberales", "category": "academic" },
          { "id": "ability.philosophiae", "category": "academic" },
          { "id": "ability.single_weapon", "category": "martial" },
          { "id": "ability.brawl", "category": "general", "combat_ability": true },
          { "id": "ability.ride", "category": "general" }
        ] }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }, { "score": 10, "total_xp": 55 },
            { "score": 13, "total_xp": 91 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.muto", "art_type": "technique" },
            { "id": "art.perdo", "art_type": "technique" },
            { "id": "art.corpus", "art_type": "form" },
            { "id": "art.ignem", "art_type": "form" },
            { "id": "art.terram", "art_type": "form" },
            { "id": "art.vim", "art_type": "form" }
          ]
        }"#;
        let spells = r#"{ "spells": [
          { "id": "spell.pilum_of_fire", "technique": "art.creo", "form": "art.ignem",
            "level": 20, "ritual": false },
          { "id": "spell.wizards_boost_form", "technique": "art.muto", "form": "art.vim",
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }] }
        ] }"#;
        let equipment = r#"{
          "weapons": [
            { "id": "weapon.long_sword", "kind": "melee", "init_mod": 2, "attack_mod": 4,
              "defense_mod": 1, "damage_mod": 6, "min_strength": 0, "load": 1,
              "ability": "ability.single_weapon" },
            { "id": "weapon.great_sword", "kind": "melee", "init_mod": 2, "attack_mod": 5,
              "defense_mod": 2, "damage_mod": 9, "min_strength": 0, "load": 2,
              "two_handed": true, "ability": "ability.single_weapon" },
            { "id": "weapon.dodge", "kind": "melee", "init_mod": 0, "defense_mod": 0,
              "load": 0, "ability": "ability.brawl", "body_attack": true }
          ],
          "shields": [
            { "id": "shield.round", "init_mod": 0, "attack_mod": 0, "defense_mod": 2,
              "load": 1, "min_strength": 0 }
          ],
          "armor": [
            { "id": "armor.leather_scale", "protection": 3, "load": 1 }
          ]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            houses: None,
            mythic_types: None,
            spells: Some(spells),
            spell_mastery_abilities: None,
            equipment: Some(equipment),
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
            parameter_catalogues: None,
        })
        .unwrap()
    }

    fn magus() -> Entity {
        Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    fn grog() -> Entity {
        Entity::new(
            EntityKind::Character,
            Id::new("grog"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    fn set_char(e: &mut Entity, c: Characteristic, v: i8) {
        e.characteristics.insert(c, v);
    }

    fn find_casting<'a>(totals: &'a [CastingTotal], te: &str, fo: &str) -> &'a CastingTotal {
        totals
            .iter()
            .find(|t| t.technique.as_str() == te && t.form.as_str() == fo)
            .expect("casting cell present")
    }

    /// A magus whose Creo Corpus Lab Total is 35: Int 3 + Magic Theory 4 + Creo 10
    /// + Corpus 13 + Aura 5. The shared setup for every longevity-hint test.
    fn longevity_magus() -> Entity {
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 13,
            },
        ];
        e.aura = 5;
        e
    }

    fn ritual(source: LongevitySource, bonus: Option<i8>) -> Option<LongevityRitual> {
        Some(LongevityRitual {
            source,
            bonus,
            focus: String::new(),
        })
    }

    /// A self-made ritual's bonus is the *stored* one, passed straight through: the
    /// number was frozen by the Lab Total of the season the ritual was made
    /// (ArMDE:10670), so the engine must not overwrite it with today's derivation.
    #[test]
    fn self_made_longevity_passes_through_entered_bonus() {
        let rs = ruleset();
        let mut e = longevity_magus();
        // The Lab Total is 35 (hint would suggest 7) — the stored 3 must win.
        e.longevity_ritual = ritual(LongevitySource::SelfMade, Some(3));
        let lb = longevity_bonus(&e, &rs).expect("has ritual");
        assert_eq!(lb.source, LongevitySource::SelfMade);
        assert_eq!(lb.bonus, 3, "the entered bonus, not the derived 7");
        assert!(lb.entered);
        assert_eq!(lb.hint.expect("self-made gets a hint").suggested_bonus, 7);
    }

    /// An External ritual passes the entered bonus through and gets **no** hint —
    /// its bonus came from another magus's Lab Total (ArMDE:10672), which this
    /// character sheet does not know.
    #[test]
    fn external_longevity_passes_through_entered_bonus() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.longevity_ritual = ritual(LongevitySource::External, Some(6));
        let lb = longevity_bonus(&e, &rs).expect("has ritual");
        assert_eq!(lb.source, LongevitySource::External);
        assert_eq!(lb.bonus, 6);
        assert!(lb.entered);
        assert_eq!(lb.hint, None, "no suggestion for someone else's ritual");
    }

    /// No bonus entered yet: `bonus` reads 0 but `entered` is false, so the UI can
    /// distinguish "not filled in" from a deliberate 0.
    #[test]
    fn unentered_longevity_bonus_is_zero_and_not_entered() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let lb = longevity_bonus(&e, &rs).expect("has ritual");
        assert_eq!(lb.bonus, 0);
        assert!(!lb.entered, "0 is a placeholder here, not a claim");
    }

    /// The hint: "+1 bonus for every five points or fraction of Creo Corpus Lab
    /// Total" (ArMDE:10662) — 35 → ceil(35/5) = 7, matching the book's worked
    /// example (ArMDE:2488, :2573).
    #[test]
    fn self_made_longevity_hint_is_lab_total_over_five_rounded_up() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("self-made gets a hint");
        assert_eq!(hint.lab_total, 35);
        assert_eq!(hint.suggested_bonus, 7);
        assert!(!hint.halved);
    }

    /// A zero aura does not suppress the hint. The Lab Total takes the Aura
    /// Modifier as a plain addend (ArMDE:10276-10278), and no aura simply means no
    /// hindrance (ArMDE:17658) — 30 → ceil(30/5) = 6.
    #[test]
    fn longevity_hint_survives_a_zero_aura() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.aura = 0;
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("a zero aura still gets a hint");
        assert_eq!(hint.lab_total, 30);
        assert_eq!(hint.suggested_bonus, 6);
    }

    /// A negative aura is a plain addend too, lowering the Lab Total: 35 − 5 − 3 =
    /// 27 → ceil(27/5) = 6 (ArMDE:10276-10278).
    #[test]
    fn negative_aura_lowers_the_longevity_hint() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.aura = -3;
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(hint.lab_total, 27);
        assert_eq!(hint.suggested_bonus, 6);
    }

    /// `Entity::aura` is only clamped to `AURA_MODIFIER_MIN`..=`AURA_MODIFIER_MAX`
    /// by `types.rs::Entity::normalize`; a value that reaches
    /// `creo_corpus_lab_total` before that pass runs (e.g. a freshly deserialized
    /// save under `ValidationMode::Silent`, which still computes derived totals —
    /// "one evaluation path") must not overflow the bare `i32` sum. Saturates
    /// instead of wrapping.
    #[test]
    fn an_out_of_range_aura_saturates_the_longevity_hint_instead_of_overflowing() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.aura = i32::MAX; // deliberately unclamped — normalize() was not called
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(
            hint.lab_total,
            i32::MAX,
            "saturates at i32::MAX rather than wrapping negative"
        );
    }

    /// Deficient Creo halves the Lab Total the hint reads (ArMDE:5909-5915):
    /// 35 → 17 → ceil(17/5) = 4, flagged `halved`.
    #[test]
    fn deficient_creo_halves_the_longevity_hint() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.selections = vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("art".to_string(), Id::new("art.creo"))]),
        )];
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(hint.lab_total, 17);
        assert_eq!(hint.suggested_bonus, 4);
        assert!(hint.halved);
    }

    /// Difficult Longevity Ritual: "Anyone (including yourself) creating a Longevity
    /// Ritual for you must halve their Lab Total" (ArMDE:5962-5964) — 35 → 17 → 4.
    #[test]
    fn difficult_longevity_ritual_halves_the_hint() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.selections = vec![Selection::new(Id::new("flaw.difficult_longevity_ritual"))];
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(hint.lab_total, 17);
        assert_eq!(hint.suggested_bonus, 4);
        assert!(hint.halved);
    }

    /// The two halvings compound (an inference — neither Flaw carves out the other):
    /// Deficient Corpus + Difficult Longevity Ritual → 35 → 17 → 8 → ceil(8/5) = 2.
    #[test]
    fn both_longevity_halvings_stack() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.selections = vec![
            Selection::with_params(
                Id::new("flaw.deficient_form"),
                BTreeMap::from([("form".to_string(), Id::new("art.corpus"))]),
            ),
            Selection::new(Id::new("flaw.difficult_longevity_ritual")),
        ];
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(hint.lab_total, 8, "floor(floor(35/2)/2)");
        assert_eq!(hint.suggested_bonus, 2);
        assert!(hint.halved);
    }

    /// A non-positive Lab Total suggests no bonus at all — "every five points" has
    /// no meaning below one point (ArMDE:10662).
    #[test]
    fn non_positive_longevity_lab_total_suggests_no_bonus() {
        let rs = ruleset();
        let mut e = magus();
        // No Int, no Magic Theory, no Arts; a −6 aura drives the total negative.
        e.aura = -6;
        e.longevity_ritual = ritual(LongevitySource::SelfMade, None);
        let hint = longevity_bonus(&e, &rs)
            .expect("has ritual")
            .hint
            .expect("has a hint");
        assert_eq!(hint.lab_total, -6);
        assert_eq!(hint.suggested_bonus, 0);
    }

    /// Masterpiece: the best (Technique, Form) Lab Total bounds the lesser
    /// enchanted item the magus could make — level ≤ Lab Total ÷ 2 (ArMDE:10410).
    /// Int 3 + Magic Theory 4 + Creo 10 + Corpus 13 + Aura 5 = 35 → cap 17.
    #[test]
    fn masterpiece_cap_is_best_lab_total_halved() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 13,
            },
        ];
        e.aura = 5;
        e.selections = vec![Selection::new(Id::new("virtue.masterpiece"))];
        let cap = masterpiece_item_cap(&lab_totals(&e, &rs), &e, &rs).expect("has masterpiece");
        assert_eq!(cap.lab_total, 35);
        assert_eq!(cap.cap, 17);
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.form.as_str(), "art.corpus");
    }

    /// Without the Masterpiece Virtue there is no item cap.
    #[test]
    fn masterpiece_cap_absent_without_virtue() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        }];
        assert!(masterpiece_item_cap(&lab_totals(&e, &rs), &e, &rs).is_none());
        assert!(derived_totals(&e, &rs).masterpiece.is_none());
    }

    /// A Weak Enchanter designing his Masterpiece item is still "creating" an
    /// enchanted item (ArMDE:4476-4479:
    /// the item is designed "following the regular rules for construction of such
    /// a device"), so the cap must derive from the halved `enchanting` figure
    /// (ArMDE:7060-7063), not the un-halved `total` — otherwise the cap is double what
    /// the rules allow. Same figures as `masterpiece_cap_is_best_lab_total_halved`
    /// (total 35) but with Weak Enchanter added: enchanting = halve(35) = 17, so
    /// the reported lab_total is 17 and the cap is halve(17) = 8, not 35 / 17.
    #[test]
    fn masterpiece_cap_halves_for_weak_enchanter() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 13,
            },
        ];
        e.aura = 5;
        e.selections = vec![
            Selection::new(Id::new("virtue.masterpiece")),
            Selection::new(Id::new("flaw.weak_enchanter")),
        ];
        let cap = masterpiece_item_cap(&lab_totals(&e, &rs), &e, &rs).expect("has masterpiece");
        assert_eq!(cap.lab_total, 17);
        assert_eq!(cap.cap, 8);
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.form.as_str(), "art.corpus");
    }

    /// A magus with a talisman: its capacity in pawns of Vim vis is his highest
    /// Technique + his highest Form (ArMDE:10619). Creo 10 / Perdo 4 and Corpus 12 /
    /// Ignem 8 → Creo + Corpus = 22, with both contributing scores surfaced so the
    /// UI needs no arithmetic.
    #[test]
    fn talisman_capacity_is_highest_technique_plus_highest_form() {
        let rs = ruleset();
        let mut e = magus();
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.perdo"),
                score: 4,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 12,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 8,
            },
        ];
        e.talisman = Some(Talisman::default());
        let cap = talisman_capacity(&e, &rs).expect("a talisman has a capacity");
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.form.as_str(), "art.corpus");
        assert_eq!(cap.technique_score, 10);
        assert_eq!(cap.form_score, 12);
        assert_eq!(cap.pawns, 22);
        // Reaches the aggregated read-out for a magus.
        assert_eq!(
            derived_totals(&e, &rs).talisman_capacity.map(|c| c.pawns),
            Some(22)
        );
    }

    /// No talisman, no capacity read-out — and a non-magus never gets one even if a
    /// hand-edited save carries a talisman.
    #[test]
    fn talisman_capacity_absent_without_a_talisman_or_for_a_non_magus() {
        let rs = ruleset();
        let mut e = magus();
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 10,
        }];
        assert!(talisman_capacity(&e, &rs).is_none());
        assert!(derived_totals(&e, &rs).talisman_capacity.is_none());

        let mut g = grog();
        g.talisman = Some(Talisman::default());
        assert!(derived_totals(&g, &rs).talisman_capacity.is_none());
    }

    /// The capacity reads the per-Art **effective scores**, not the best Lab Total
    /// pair: it "depends on the power of the magus" (ArMDE:10619), and a Deficient
    /// Technique halves *totals*, never the Art score itself. Discriminating: with
    /// Deficient Creo the best Lab Total moves to Perdo, while the capacity stays on
    /// Creo + Corpus.
    #[test]
    fn talisman_capacity_ignores_the_deficient_art_halving() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.perdo"),
                score: 4,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 12,
            },
        ];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("art".to_string(), Id::new("art.creo"))]),
        )];
        e.talisman = Some(Talisman::default());

        // The best Lab Total is no longer a Creo cell: Creo+Corpus 29 halves to 14,
        // while Perdo+Corpus is an unhalved 23.
        let best = lab_totals(&e, &rs)
            .into_iter()
            .max_by_key(|lt| lt.total)
            .expect("lab grid populated");
        assert_eq!(best.technique.as_str(), "art.perdo");
        assert_eq!(best.total, 23);

        // The capacity is unmoved.
        let cap = talisman_capacity(&e, &rs).expect("a talisman has a capacity");
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.pawns, 22);
    }

    /// A magus who has bought no Arts still gets a read-out, at 0 pawns, naming the
    /// alphabetically first Technique/Form — the tie is broken deterministically, not
    /// by iteration luck.
    #[test]
    fn talisman_capacity_is_zero_and_deterministic_without_art_scores() {
        let rs = ruleset();
        let mut e = magus();
        e.talisman = Some(Talisman::default());
        let cap = talisman_capacity(&e, &rs).expect("a talisman has a capacity");
        assert_eq!(cap.pawns, 0);
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.form.as_str(), "art.corpus");
    }

    /// Gerda #9. Three panels of one character sheet answer "your best Technique
    /// and Form": the talisman capacity, the Masterpiece cap and the familiar
    /// binding. `talisman_capacity` breaks a tie by taking the **first** strict
    /// maximum and documents why — "so the read-out never flickers between equal
    /// Arts" (`familiar.rs::highest_art`). The other two reached for `max_by_key`,
    /// which returns the **last** maximum, so on a magus who has bought no Arts —
    /// every cell of the grid identical — one panel said Creo/Corpus while the
    /// other two said Perdo/Vim, and the answer moved as soon as any single Art
    /// was bought.
    ///
    /// The pair itself is arbitrary on an all-zero grid; what is not arbitrary is
    /// that the three agree, which is why this asserts them against each other as
    /// well as against the alphabetically-first pair.
    #[test]
    fn the_three_best_art_readouts_break_a_tie_the_same_way() {
        let rs = ruleset();
        let mut e = magus();
        // No Art scores at all: every Lab Total cell is identical, so every cell
        // is a maximum and only the tie-break decides the answer.
        e.talisman = Some(Talisman::default());
        e.familiar = Some(statblock_familiar());
        e.selections = vec![Selection::new(Id::new("virtue.masterpiece"))];

        let talisman = talisman_capacity(&e, &rs).expect("a talisman has a capacity");
        let grid = lab_totals(&e, &rs);
        let masterpiece = masterpiece_item_cap(&grid, &e, &rs).expect("Masterpiece is present");
        let familiar = familiar_readout(&grid, &e).expect("a familiar has a read-out");

        assert_eq!(
            (masterpiece.technique.as_str(), masterpiece.form.as_str()),
            (talisman.technique.as_str(), talisman.form.as_str()),
            "the Masterpiece cap must name the same pair as the talisman capacity"
        );
        assert_eq!(
            (
                familiar.binding.technique.as_str(),
                familiar.binding.form.as_str()
            ),
            (talisman.technique.as_str(), talisman.form.as_str()),
            "the familiar binding must name the same pair as the talisman capacity"
        );
        // And that shared answer is the alphabetically first pair, as the
        // documented first-strict-maximum fold gives.
        assert_eq!(talisman.technique.as_str(), "art.creo");
        assert_eq!(talisman.form.as_str(), "art.corpus");
    }

    /// A ruleset with the given type profile and Art catalogue, for the empty and
    /// **one-sided** Art catalogues the core fixture cannot express. Catalogue size
    /// and shape are data, never code (CLAUDE.md), so the engine must cope with a
    /// ruleset that defines no Art of a class instead of assuming the core rules'
    /// 5 × 10 grid.
    fn ruleset_with_profile_and_arts(type_profiles: &str, arts: Option<&str>) -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: "[]",
            type_profiles,
            abilities: None,
            arts,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
            parameter_catalogues: None,
        })
        .unwrap()
    }

    /// The magus profile of [`ruleset`], on its own.
    const MAGUS_PROFILE: &str = r#"[
      { "id": "magus", "hermetically_trained": true, "order_member": true,
        "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "hermetic"], "forbidden_categories": [],
        "required_traits": [], "forbidden_traits": [], "gift_policy": "required",
        "gift_categories": [], "creation_phases": ["concept"] }
    ]"#;

    /// A magus-capable ruleset that ships **no Art catalogue at all** — neither
    /// Technique nor Form. `validate_integrity` demands the engine-dereferenced Arts
    /// (Creo, Corpus) only once a ruleset ships Arts, so this is a legal ruleset.
    fn magus_ruleset_without_arts() -> Ruleset {
        ruleset_with_profile_and_arts(MAGUS_PROFILE, None)
    }

    /// An Art catalogue with **no Technique** yields no capacity at all — distinct
    /// from `talisman_capacity_is_zero_and_deterministic_without_art_scores`, where
    /// the full catalogue is present and only the magus's *scores* are absent.
    /// `highest_art` folds an empty id list to `None`, so the capacity is `None`
    /// rather than a half-answer naming an Art the ruleset does not define.
    #[test]
    fn talisman_capacity_is_absent_when_the_art_catalogue_defines_no_technique() {
        let rs = magus_ruleset_without_arts();
        assert!(
            rs.art_ids_of(ArtType::Technique).is_empty(),
            "fixture defines no Technique"
        );
        let mut e = magus();
        e.talisman = Some(Talisman::default());
        assert!(talisman_capacity(&e, &rs).is_none());
        assert!(derived_totals(&e, &rs).talisman_capacity.is_none());
    }

    /// The sibling branch: Techniques defined, but **no Form**. Such a catalogue can
    /// only belong to a ruleset with no magus profile — `validate_integrity` would
    /// otherwise insist on Corpus, a Form — so the fixture declares a non-magus
    /// profile and `talisman_capacity` is exercised directly (`derived_totals` gates
    /// the capacity on `is_magus`).
    #[test]
    fn talisman_capacity_is_absent_when_the_art_catalogue_defines_no_form() {
        let grog_profile = r#"[
          { "id": "grog", "hermetically_trained": false, "order_member": false,
            "budget": { "virtue_points": 3, "flaw_points": 3 },
            "permitted_categories": ["general"], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "gift_policy": "forbidden",
            "gift_categories": [], "creation_phases": ["concept"] }
        ]"#;
        let techniques_only = r#"{
          "advancement": [{ "score": 1, "total_xp": 1 }],
          "arts": [{ "id": "art.creo", "art_type": "technique" }]
        }"#;
        let rs = ruleset_with_profile_and_arts(grog_profile, Some(techniques_only));
        assert!(!rs.art_ids_of(ArtType::Technique).is_empty());
        assert!(
            rs.art_ids_of(ArtType::Form).is_empty(),
            "one-sided catalogue"
        );
        let mut e = grog();
        e.talisman = Some(Talisman::default());
        assert!(talisman_capacity(&e, &rs).is_none());
    }

    /// A familiar with a Magic Might and Personality Traits, for the read-out
    /// tests. Cords 3/2/1 cost 30 + 15 + 5 = 50 points.
    fn statblock_familiar() -> Familiar {
        Familiar {
            name: "Corax".into(),
            animal: "raven".into(),
            might: Some(MightScore {
                realm: Realm::Magic,
                score: 10,
            }),
            characteristics: BTreeMap::from([(Characteristic::Int, -3)]),
            size: -4,
            personality_traits: Vec::new(),
            cord_gold: 3,
            cord_silver: 2,
            cord_bronze: 1,
            powers: vec![SupernaturalPower {
                name: "Mental communication".into(),
                level: 15,
                penetration: 0,
            }],
        }
    }

    /// Cord scores are bought off a fixed 5-entry curve — +1 costs 5, +2 15, +3 30,
    /// +4 50, +5 75 — and the read-out is the **total** across all three cords
    /// (ArMDE:10836).
    #[test]
    fn cord_points_spent_follows_the_cord_cost_curve() {
        let mut f = Familiar::default();
        assert_eq!(cord_points_spent(&f), 0, "0/0/0 costs nothing");

        f.cord_gold = 3;
        f.cord_silver = 2;
        f.cord_bronze = 1;
        assert_eq!(cord_points_spent(&f), 50, "30 + 15 + 5");

        f.cord_gold = 5;
        f.cord_silver = 5;
        f.cord_bronze = 5;
        assert_eq!(cord_points_spent(&f), 225, "3 x 75, the maximum cords");
    }

    /// A cord score above the curve's top (+5 is the maximum, ArMDE:10836) is
    /// **clamped**, not indexed: the cord fields are `u8`, so a hand-edited or legacy
    /// save can carry any value up to 255, and a raw index would panic inside the
    /// derived-totals command and take the whole read-out panel down.
    #[test]
    fn cord_points_spent_clamps_a_score_above_the_curve() {
        let f = Familiar {
            cord_gold: 255,
            cord_silver: 6,
            cord_bronze: 0,
            ..Default::default()
        };
        assert_eq!(cord_points_spent(&f), 150, "both clamp to +5 = 75 each");
    }

    /// The +5 cord maximum (ArMDE:10836) is enforced in **one** place, so every
    /// consumer of a cord score reports the same number. A hand-edited save
    /// carrying `cord_bronze: 255` must not read as "75 points spent" on the
    /// familiar panel while Soak and the Longevity Bronze-cord note both claim
    /// +255 — one entered value, three contradictory figures.
    #[test]
    fn an_out_of_range_bronze_cord_reads_the_same_for_every_consumer() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.longevity_ritual = ritual(LongevitySource::SelfMade, Some(3));
        e.familiar = Some(Familiar {
            name: "Corax".to_string(),
            cord_bronze: 255,
            ..Default::default()
        });

        let familiar = e.familiar.as_ref().expect("familiar present");
        assert_eq!(cord_points_spent(familiar), 75, "clamped to +5 = 75 points");

        let bronze_soak = soak(&e, &rs)
            .addends
            .into_iter()
            .find(|a| a.label == "bronze_cord")
            .expect("Soak lists the Bronze cord")
            .value;
        assert_eq!(bronze_soak, 5, "Soak takes the clamped +5, not the raw 255");

        assert_eq!(
            longevity_bonus(&e, &rs).expect("has ritual").bronze_cord,
            5,
            "the aging-resistance note takes the clamped +5 too"
        );
    }

    /// The three **entity-level** Bronze-cord read-outs all clamp at the same +5
    /// maximum (ArMDE:10836), because they share one accessor
    /// ([`bronze_cord_bonus`]) which itself routes through [`cord_score`]. The
    /// clamp must not be bypassable by any single path: a hand-edited save
    /// carrying `cord_bronze: 255` reads +5 through the accessor, +5 in Soak, and
    /// +5 on the Longevity Ritual's aging-resistance note.
    #[test]
    fn all_three_cord_readouts_clamp_at_the_same_maximum() {
        let rs = ruleset();
        let mut e = longevity_magus();
        e.longevity_ritual = ritual(LongevitySource::SelfMade, Some(3));
        e.familiar = Some(Familiar {
            name: "Corax".to_string(),
            cord_bronze: 255,
            ..Default::default()
        });

        let clamped = i32::from(MAX_CORD_SCORE);
        assert_eq!(
            bronze_cord_bonus(&e),
            clamped,
            "the accessor clamps the raw 255 to +5"
        );

        let bronze_soak = soak(&e, &rs)
            .addends
            .into_iter()
            .find(|a| a.label == "bronze_cord")
            .expect("Soak lists the Bronze cord")
            .value;
        assert_eq!(bronze_soak, clamped, "Soak reads the same clamped +5");

        assert_eq!(
            longevity_bonus(&e, &rs).expect("has ritual").bronze_cord,
            clamped,
            "the aging-resistance note reads the same clamped +5"
        );
    }

    /// An entity with no familiar has no Bronze cord, and the accessor says 0 —
    /// not a panic and not an absent addend.
    #[test]
    fn bronze_cord_bonus_is_zero_without_a_familiar() {
        let mut e = longevity_magus();
        e.familiar = None;
        assert_eq!(bronze_cord_bonus(&e), 0);
    }

    /// The bonding level is "25 plus the familiar's Magic Might plus 5 times its
    /// Size", and a negative Size **reduces** it — the book's own worked example:
    /// Size -2 and Magic Might 10 bind as a level 25 enchantment (ArMDE:10824,
    /// :10828).
    #[test]
    fn familiar_binding_level_folds_in_a_negative_size() {
        let book_example = Familiar {
            might: Some(MightScore {
                realm: Realm::Magic,
                score: 10,
            }),
            size: -2,
            ..Default::default()
        };
        assert_eq!(familiar_binding_level(&book_example), 25);

        let raven = statblock_familiar(); // Might 10, Size -4
        assert_eq!(familiar_binding_level(&raven), 25 + 10 - 20);
    }

    /// A familiar with no entered Might contributes 0 to the level rather than
    /// suppressing the read-out; the panel says "no Magic Might entered".
    #[test]
    fn familiar_binding_level_without_might_counts_might_as_zero() {
        let f = Familiar {
            size: 1,
            ..Default::default()
        };
        assert_eq!(familiar_binding_level(&f), 30);
        assert_eq!(familiar_binding_level(&Familiar::default()), 25);
    }

    /// A Focus Power's Initiative is "the character's Quickness – the maximum
    /// magnitude of the effect" (ArMDE:3899), and a magnitude "is equal to the
    /// level divided by five, rounded up" (`ArMDE:9097`) — the only sourced
    /// level→magnitude rule the book gives. Activation costs "one Fatigue level …
    /// for an effect of level 25 or lower, two Fatigue levels … if the effect has a
    /// level of 26 to 50, and three for 51 to 75" (`ArMDE:3901`); the book stops at
    /// 75, so above it the cost is unstated rather than extrapolated.
    #[test]
    fn focus_power_lines_derive_magnitude_initiative_and_fatigue() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Qik, 2);
        e.focus_powers = vec![
            FocusPower {
                name: "Wolves".into(),
                max_level: 10,
                penetration: 5,
            },
            FocusPower {
                name: "Storms".into(),
                max_level: 51,
                penetration: 0,
            },
            FocusPower {
                name: "Past the table".into(),
                max_level: 80,
                penetration: 0,
            },
        ];

        let lines = focus_power_lines(&e, &rs);
        assert_eq!(lines.len(), 3);

        assert_eq!(lines[0].name, "Wolves");
        assert_eq!(lines[0].magnitude, 2); // 10 ÷ 5
        assert_eq!(lines[0].initiative, 0); // Qik 2 − 2
        assert_eq!(lines[0].fatigue_levels, Some(1)); // level ≤ 25

        assert_eq!(lines[1].magnitude, 11); // 51 ÷ 5, rounded UP
        assert_eq!(lines[1].initiative, -9); // Qik 2 − 11
        assert_eq!(lines[1].fatigue_levels, Some(3)); // 51–75

        assert_eq!(lines[2].fatigue_levels, None); // above 75 the book is silent

        // No focus powers, no lines.
        assert!(focus_power_lines(&magus(), &rs).is_empty());
    }

    /// The read-out has to reach the panel, so the whole-character totals carry the
    /// focus-power lines; a character with none carries an empty list.
    #[test]
    fn derived_totals_carry_the_focus_power_lines() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Qik, 1);
        e.focus_powers = vec![FocusPower {
            name: "Wolves".into(),
            max_level: 10,
            penetration: 5,
        }];

        let out = derived_totals(&e, &rs);
        assert_eq!(out.focus_powers.len(), 1);
        assert_eq!(out.focus_powers[0].initiative, -1); // Qik 1 − magnitude 2

        assert!(derived_totals(&magus(), &rs).focus_powers.is_empty());
    }

    /// D56/A0's sub-slice-2 headline case: a grog (untrained profile) holding a
    /// **test-only fixture** selection that carries `Effect::ConfersHermeticTraining`
    /// gets the magic totals a magus gets — Lab, Casting and Penetration all
    /// populated — because `derived_totals` must key on `is_hermetically_trained`,
    /// not the bare profile flag. The real `flaw.abandoned_apprentice` carries no
    /// such effect yet (D3), so it is unaffected by this fixture.
    #[test]
    fn derived_totals_for_a_trained_non_magus_test_fixture_include_casting_lab_and_penetration() {
        let rs = ruleset();
        let mut e = grog();
        e.selections = vec![Selection::new(Id::new("flaw.test_confers_training"))];
        set_char(&mut e, Characteristic::Int, 3);
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.magic_theory"),
                parameter: None,
                specialty: None,
                score: 4,
            },
            AbilityScore {
                ability: Id::new("ability.penetration"),
                parameter: None,
                specialty: None,
                score: 4,
            },
        ];
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        }];

        let out = derived_totals(&e, &rs);
        assert!(
            out.hermetically_trained,
            "the fixture selection must union in"
        );
        assert!(!out.lab_totals.is_empty(), "lab totals populated");
        assert!(!out.casting_totals.is_empty(), "casting totals populated");
        assert!(!out.penetration.is_empty(), "penetration lines populated");

        // Without the fixture effect, the same grog gets none of them — the gate
        // genuinely switched on, it did not stand down for every grog.
        let mut untrained = grog();
        untrained.characteristics = e.characteristics.clone();
        untrained.art_scores = e.art_scores.clone();
        untrained.ability_scores = e.ability_scores.clone();
        untrained.spells = e.spells.clone();
        let untrained_out = derived_totals(&untrained, &rs);
        assert!(!untrained_out.hermetically_trained);
        assert!(untrained_out.lab_totals.is_empty());
        assert!(untrained_out.casting_totals.is_empty());
        assert!(untrained_out.penetration.is_empty());
    }

    /// Bond-invested power levels are summed for information only: "there is no
    /// limit to the number of powers which may be invested in a familiar"
    /// (ArMDE:10866), so there is no budget to compare against and no issue to
    /// raise.
    #[test]
    fn familiar_invested_power_levels_sums_with_no_budget() {
        let mut f = statblock_familiar();
        assert_eq!(familiar_invested_power_levels(&f), 15);
        f.powers.push(SupernaturalPower {
            name: "Shapechanging".into(),
            level: 25,
            penetration: 0,
        });
        assert_eq!(familiar_invested_power_levels(&f), 40);
    }

    /// The bonding Lab Total is the ordinary Lab Total shape (ArMDE:10826), so the
    /// best `(Technique, Form)` cell of the existing grid is taken — and unlike
    /// Masterpiece, a **focus** may apply here (`ArMDE:10818`), so the best cell's
    /// within-focus figure is surfaced as a separate conditional number.
    #[test]
    fn familiar_readout_takes_the_best_lab_total_and_surfaces_the_focus() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 13,
            },
        ];
        e.aura = 5;
        e.selections = vec![Selection::with_params(
            Id::new("virtue.magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("ravens"))]),
        )];
        e.familiar = Some(statblock_familiar());

        let out = familiar_readout(&lab_totals(&e, &rs), &e).expect("a familiar has a read-out");
        assert_eq!(out.binding_level, 15);
        assert_eq!(out.cord_points_spent, 50);
        assert_eq!(out.invested_power_levels, 15);

        let binding = &out.binding;
        // Best cell: Int 3 + Magic Theory 4 + Creo 10 + Corpus 13 + Aura 5 = 35.
        assert_eq!(binding.technique.as_str(), "art.creo");
        assert_eq!(binding.form.as_str(), "art.corpus");
        assert_eq!(binding.lab_total, 35);
        // Within the focus the lower Art (Creo 10) is added again.
        assert_eq!(binding.lab_total_within_focus, Some(45));
        assert!(binding.lab_total_reaches_level, "35 >= 15");
        assert!(
            !binding.cord_points_within_lab_total,
            "50 cord points overrun a Lab Total of 35"
        );
    }

    /// Without a Magical Focus the conditional within-focus figure is absent, and
    /// the two comparison flags report a Lab Total that falls short of the level and
    /// cords that overrun it.
    #[test]
    fn familiar_readout_reports_a_lab_total_that_falls_short() {
        let rs = ruleset();
        let mut e = magus();
        e.familiar = Some(statblock_familiar());
        let out = familiar_readout(&lab_totals(&e, &rs), &e).expect("a familiar has a read-out");
        assert_eq!(out.binding.lab_total, 0);
        assert_eq!(out.binding.lab_total_within_focus, None);
        assert!(!out.binding.lab_total_reaches_level, "0 < 15");
        assert!(!out.binding.cord_points_within_lab_total, "50 > 0");
    }

    /// No familiar, or a non-magus, means no read-out at all — `DerivedTotals`
    /// gates it on `is_magus` exactly as it gates the talisman capacity.
    #[test]
    fn familiar_readout_absent_without_a_familiar_or_for_a_non_magus() {
        let rs = ruleset();
        let e = magus();
        assert!(familiar_readout(&lab_totals(&e, &rs), &e).is_none());
        assert!(derived_totals(&e, &rs).familiar.is_none());

        let mut g = grog();
        g.familiar = Some(statblock_familiar());
        assert!(derived_totals(&g, &rs).familiar.is_none());

        let mut m = magus();
        m.familiar = Some(statblock_familiar());
        assert!(derived_totals(&m, &rs).familiar.is_some());
    }

    /// An empty Art catalogue makes the Lab-Total grid empty, and the read-out is
    /// then `None` — the binding numbers are meaningful only beside a Lab Total, so
    /// no partial read-out is emitted. This is the empty-*catalogue* case, not the
    /// zero-*score* case `familiar_readout_reports_a_lab_total_that_falls_short`
    /// covers (there the grid is full and the best cell is simply 0).
    #[test]
    fn familiar_readout_is_absent_when_the_art_catalogue_yields_no_lab_totals() {
        let rs = magus_ruleset_without_arts();
        let mut e = magus();
        e.familiar = Some(statblock_familiar());
        assert!(lab_totals(&e, &rs).is_empty(), "no Arts, so no grid cell");
        assert!(familiar_readout(&lab_totals(&e, &rs), &e).is_none());
        assert!(derived_totals(&e, &rs).familiar.is_none());
    }

    /// `Ruleset::art_ids_of` returns the ids of one Art class, sorted. The sort is
    /// load-bearing: `spell_level_caps` documents a canonical `(technique, form)`
    /// ordering, and a hand-built fixture's declaration order need not be sorted.
    #[test]
    fn art_ids_of_returns_sorted_ids_of_one_class() {
        let rs = ruleset();
        let techniques = rs.art_ids_of(ArtType::Technique);
        assert_eq!(
            techniques.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            vec!["art.creo", "art.muto", "art.perdo"]
        );
        let mut sorted = techniques.clone();
        sorted.sort();
        assert_eq!(techniques, sorted, "ids come back sorted");
        assert!(
            rs.art_ids_of(ArtType::Form)
                .iter()
                .all(|id| id.as_str() != "art.creo"),
            "Forms only"
        );
    }

    /// A casting total with Encumbrance and a Magical Focus: base vs within-focus,
    /// plus Method Caster's +3 formulaic (ArMDE:9089, :4524-4527, :4399-4422).
    #[test]
    fn casting_total_with_encumbrance_and_focus() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        set_char(&mut e, Characteristic::Str, 0);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.aura = 3;
        // A weapon of Load 6 → Burden 3; Strength 0 → Encumbrance 3.
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Stowed,
            specialization_applies: false,
        }];
        // Give the weapon Load 6 via a heavier item: use armor Load path instead.
        // Worn, not stowed: only equipped gear counts toward Load
        // (`combat.rs::encumbrance`), and this test needs a non-zero Encumbrance
        // to subtract from the Casting Score.
        e.equipment = vec![EquipmentSlot {
            item: Id::new("armor.leather_scale"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        // Method Caster (+3 formulaic) and a Magical Focus.
        e.selections = vec![
            Selection::new(Id::new("virtue.method_caster")),
            Selection::with_params(
                Id::new("virtue.magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("fire"))]),
            ),
        ];
        // Encumbrance: armor Load 1 → Burden 1, Strength 0 → Encumbrance 1.
        assert_eq!(encumbrance(&e, &rs).total, 1);
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        // Base formulaic = Cr10 + Ig5 + Sta2 − Enc1 + Aura3 + MethodCaster3 = 22.
        assert_eq!(cell.formulaic, 22);
        // Within focus: + lower Art (Ignem 5) = 27.
        let wf = cell.within_focus.as_ref().expect("focus present");
        assert_eq!(wf.focus_art, 5);
        assert_eq!(wf.formulaic, 27);
        // Spontaneous fatiguing (no Method Caster on spont): (22−3)/2 = 9.
        assert_eq!(cell.spontaneous_fatiguing, 9);
    }

    /// An unclamped `Entity::aura` (see the sibling longevity-hint overflow test)
    /// reaches `casting_totals`/`lab_totals` through the shared `sum()` addend
    /// helper. Must saturate, not wrap, so a hostile or pre-normalize save cannot
    /// turn into a silently-wrong (and possibly negative) Casting/Lab Total.
    #[test]
    fn an_out_of_range_aura_saturates_casting_and_lab_totals_instead_of_overflowing() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.aura = i32::MAX; // deliberately unclamped — normalize() was not called

        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(cell.formulaic, i32::MAX, "saturates rather than wraps");

        let labs = lab_totals(&e, &rs);
        let lab_cell = labs
            .iter()
            .find(|l| l.technique.as_str() == "art.creo" && l.form.as_str() == "art.ignem")
            .expect("lab cell present");
        assert_eq!(lab_cell.total, i32::MAX, "saturates rather than wraps");
    }

    /// Cluster A. The three sibling `aura = i32::MAX` tests (this module's
    /// longevity-hint, casting/lab and penetration ones) each assert **one**
    /// total against a fixture whose trailing addends are all zero — no Magical
    /// Focus, no Artes Liberales, no Philosophiae — so the plain `+` chains that
    /// sit *downstream* of the saturated `common`/`base` were never handed a
    /// non-zero term to overflow with. This one drives the **aggregator**
    /// instead, so every total that exists today and every total added later is
    /// covered by one test, and the character carries all three of the addends
    /// the per-total fixtures omit.
    ///
    /// It asserts saturation rather than a number because the interesting
    /// failure is not a wrong value: this workspace sets `overflow-checks = true`
    /// under `[profile.release]` (locked by `tests/overflow_checks_profile.rs`),
    /// so an unguarded `+` here aborts the shipped binary and takes every unsaved
    /// edit with it.
    #[test]
    fn derived_totals_with_an_unclamped_aura_saturates_rather_than_aborting() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        // The addends the per-total fixtures leave at zero: the ritual pair…
        e.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.artes_liberales"),
                parameter: None,
                specialty: None,
                score: 3,
            },
            AbilityScore {
                ability: Id::new("ability.philosophiae"),
                parameter: None,
                specialty: None,
                score: 2,
            },
        ];
        // …and a Magical Focus, which is what makes `focus_add` and the Lab
        // Total's `within_focus` non-zero.
        e.selections = vec![Selection::with_params(
            Id::new("virtue.magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("fire"))]),
        )];
        e.aura = i32::MAX; // deliberately unclamped — normalize() was not called

        let totals = derived_totals(&e, &rs);

        let cell = find_casting(&totals.casting_totals, "art.creo", "art.ignem");
        assert_eq!(cell.formulaic, i32::MAX, "saturates rather than aborting");
        assert_eq!(
            cell.ritual,
            i32::MAX,
            "the ritual addends must not overflow"
        );
        let wf = cell.within_focus.as_ref().expect("focus present");
        assert_eq!(wf.formulaic, i32::MAX);
        assert_eq!(wf.ritual, i32::MAX);

        let lab_cell = totals
            .lab_totals
            .iter()
            .find(|l| l.technique.as_str() == "art.creo" && l.form.as_str() == "art.ignem")
            .expect("lab cell present");
        assert_eq!(lab_cell.total, i32::MAX);
        assert_eq!(
            lab_cell.within_focus.expect("focus present"),
            i32::MAX,
            "the within-focus Lab Total must not overflow either"
        );
    }

    /// Cluster A, the other end of the range. The aura field accepts the whole
    /// `i32` range in both directions, and the non-standard-casting lines add a
    /// **negative** residual to an already-saturated total
    /// (`silent = formulaic + voice_penalty`, the voice penalty a flat -10 for
    /// any magus without Deft Form). So a saturated `i32::MIN` aborts there for the
    /// mirror-image reason a saturated `i32::MAX` aborts on the ritual line: the
    /// addends' sign is what decides which end overflows, never a guard.
    #[test]
    fn derived_totals_with_a_minimally_unclamped_aura_saturates_rather_than_aborting() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.aura = i32::MIN; // deliberately unclamped — normalize() was not called

        let totals = derived_totals(&e, &rs);

        // The bought-Art cell has enough headroom that nothing saturates:
        // Cr10 + Ig5 + Sta2 + aura = `i32::MIN + 17`, and the -10 voice / -5
        // gesture residuals still fit. Pinned so the test cannot be read as
        // "everything clamps".
        let bought = find_casting(&totals.casting_totals, "art.creo", "art.ignem");
        assert_eq!(bought.formulaic, i32::MIN + 17);
        assert_eq!(bought.non_standard.silent, i32::MIN + 7);
        assert_eq!(bought.non_standard.silent_and_still, i32::MIN + 2);

        // The cell that actually underflows is one with **no** Arts bought:
        // Sta2 + aura = `i32::MIN + 2`, and the two residuals take it past the
        // floor. This is the cell the abort came from, and there is one for
        // every unbought Technique/Form pair on a real character sheet.
        let unbought = find_casting(&totals.casting_totals, "art.muto", "art.corpus");
        assert_eq!(unbought.formulaic, i32::MIN + 2);
        assert_eq!(
            unbought.non_standard.silent,
            i32::MIN,
            "the -10 voice penalty must saturate, not underflow"
        );
        assert_eq!(unbought.non_standard.still, i32::MIN);
        assert_eq!(unbought.non_standard.silent_and_still, i32::MIN);
    }

    /// Cluster A, the whole route. The save file is this project's declared
    /// hostile-input surface, and the two tests above reach the derived layer by
    /// assigning `e.aura` directly — which is the *frontend's* door, not the
    /// file's. Nothing exercised `load_entity_migrating` → `derived_totals` as one
    /// flow, which is literally File → Open, so the clamp that now sits on the load
    /// path had no test standing behind it at the point where it matters.
    ///
    /// This asserts the stronger post-clamp property: a crafted aura does not merely
    /// fail to abort, it produces a *correct* sheet, because the value is brought
    /// into the rules range before any total is computed. A saturated `i32::MAX`
    /// read-out would be a wrong character sheet, which this project rates as a
    /// product-integrity failure rather than a display quirk.
    #[test]
    fn a_crafted_aura_in_a_save_is_clamped_before_any_total_is_derived() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.selections = vec![Selection::with_params(
            Id::new("virtue.magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("fire"))]),
        )];
        e.aura = i32::MAX;

        // Through the real load door, because that is where the clamp lives.
        let loaded = crate::load_entity_migrating(
            &serde_json::to_string(&e).unwrap(),
            crate::DEFAULT_SAGA_YEAR,
            &rs,
            &BTreeMap::new(),
        )
        .unwrap()
        .entity;
        assert_eq!(
            loaded.aura,
            crate::types::AURA_MODIFIER_MAX,
            "the load path clamps before the engine ever sees the value"
        );

        let totals = derived_totals(&loaded, &rs);
        let cell = find_casting(&totals.casting_totals, "art.creo", "art.ignem");
        // Cr10 + Ig5 + Sta2 - Enc0 + Aura10 = 27 — a real number, not i32::MAX.
        assert_eq!(cell.formulaic, 27);
        // …+ Artes Liberales 3 + Philosophiae 0 on the ritual line.
        assert_eq!(cell.ritual, 30);
    }

    /// A Deficient Technique halves every casting total using it (ArMDE:5913-5915).
    #[test]
    fn deficient_technique_halves_casting_total() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 0);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 4,
            },
        ];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("art".into(), Id::new("art.creo"))]),
        )];
        let totals = casting_totals(&e, &rs);
        let creo = find_casting(&totals, "art.creo", "art.ignem");
        // Cr10 + Ig4 = 14, halved → 7.
        assert!(creo.deficient);
        assert_eq!(creo.formulaic, 7);
        // Perdo is not deficient: Pe0 + Ig4 = 4, not halved.
        let perdo = find_casting(&totals, "art.perdo", "art.ignem");
        assert!(!perdo.deficient);
        assert_eq!(perdo.formulaic, 4);
    }

    /// With no relevant Virtue, casting with no voice takes −10 and with no
    /// gestures −5 off the Formulaic total (ArMDE:9243-9245); combined −15.
    ///
    /// The three figures are written out **absolutely** rather than as
    /// `formulaic - N`. A relative assertion here is invariant under any
    /// transform applied to the base — including the Deficient-Art halving —
    /// so it would agree with `halve(score) + p` and `halve(score + p)` alike;
    /// that is how the round-4 ordering defect stayed green. Cr 10 + Ig 5 +
    /// Sta 2 = 17, with no halving on this Deficiency-free fixture.
    #[test]
    fn non_standard_casting_penalties_without_virtue() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(cell.formulaic, 17);
        let nc = &cell.non_standard;
        assert_eq!(nc.voice_penalty, -10);
        assert_eq!(nc.gesture_penalty, -5);
        assert_eq!(nc.silent, 7);
        assert_eq!(nc.still, 12);
        assert_eq!(nc.silent_and_still, 2);
        assert!(!nc.deft_form);
    }

    /// A breakdown promises to *explain* the number it sits beside, and
    /// completeness is the whole of that promise — so the property worth
    /// stating is about the addend **list**, not any addend in it. Every other
    /// `addends` assertion in this file pulls one entry out by label and checks
    /// its value, which passes identically whether the list is complete or
    /// missing a term.
    ///
    /// This is a **cross-site** assertion: it walks every shipped cell of both
    /// `Addend`-bearing grids rather than one worked example, so a term added
    /// outside a list at either site reds here even though the site's own
    /// per-addend tests stay green. It is aimed at the two grids where the
    /// property is non-trivial; `SoakTotal` and the penetration total compute
    /// `total = sum(&addends)` by construction, so asserting it there would be
    /// a tautology.
    ///
    /// **Two deliberate exclusions**, both transforms *of* the sum rather than
    /// addends: the Deficient-Art halving and the within-focus double. The
    /// fixture therefore carries neither, and does carry a flat
    /// `casting_total_mod` **and** a flat `lab_total_mod`, which are exactly
    /// the terms that can go missing from a list.
    #[test]
    fn every_shipped_breakdown_accounts_for_the_total_it_explains() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        set_char(&mut e, Characteristic::Int, 3);
        e.aura = 2;
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 7,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.selections = vec![
            // +3 to Formulaic and Ritual only — so a single shared casting_mod
            // addend could not be right for every column.
            Selection::new(Id::new("virtue.method_caster")),
            // +3 to every Lab Total.
            Selection::new(Id::new("virtue.inventive_genius")),
        ];

        let addend = |addends: &[Addend], label: &str| -> i32 {
            addends
                .iter()
                .find(|a| a.label == label)
                .unwrap_or_else(|| panic!("breakdown carries a {label} addend"))
                .value
        };

        let lab = lab_totals(&e, &rs);
        assert!(!lab.is_empty(), "the fixture ships a lab grid");
        for cell in &lab {
            assert!(
                !cell.deficient,
                "fixture is Deficiency-free by construction"
            );
            assert_eq!(
                sum(&cell.addends),
                cell.total,
                "lab breakdown for {}/{} does not sum to the total it explains",
                cell.technique,
                cell.form
            );
        }

        let casting = casting_totals(&e, &rs);
        assert!(!casting.is_empty(), "the fixture ships a casting grid");
        for cell in &casting {
            assert!(
                !cell.deficient,
                "fixture is Deficiency-free by construction"
            );
            let common = sum(&cell.addends);
            assert_eq!(
                common + addend(&cell.casting_mod_addends, "casting_mod_formulaic"),
                cell.formulaic,
                "formulaic breakdown for {}/{} does not sum to the total it explains",
                cell.technique,
                cell.form
            );
            assert_eq!(
                common
                    + sum(&cell.ritual_addends)
                    + addend(&cell.casting_mod_addends, "casting_mod_ritual"),
                cell.ritual,
                "ritual breakdown for {}/{} does not sum to the total it explains",
                cell.technique,
                cell.form
            );
            // The spontaneous cells are the same sum divided, so the breakdown
            // explains the base they are derived from.
            let spont_base = common + addend(&cell.casting_mod_addends, "casting_mod_spontaneous");
            assert_eq!(
                spont_base.div_euclid(2),
                cell.spontaneous_fatiguing,
                "fatiguing spontaneous for {}/{} does not follow its breakdown",
                cell.technique,
                cell.form
            );
            assert_eq!(
                spont_base.div_euclid(5),
                cell.spontaneous_non_fatiguing,
                "non-fatiguing spontaneous for {}/{} does not follow its breakdown",
                cell.technique,
                cell.form
            );
        }
    }

    /// The Words/Gestures modifier is a penalty **to the Casting Score**
    /// (ArMDE:9236: "Increased subtlety gives a penalty to the casting score";
    /// ArMDE:9247 repeats it), and a Deficient Art halves the *total* that
    /// score feeds (ArMDE:5911: "Almost all totals (including Casting Totals
    /// and Lab Totals …) to which a particular Form is added are halved";
    /// ArMDE:9103 makes the Casting Total = Casting Score + die). So the
    /// penalty is **inside** the halving: `halve(score + penalty)`, never
    /// `halve(score) + penalty`.
    ///
    /// Every number here is written out **absolutely** rather than relative to
    /// `cell.formulaic`, and that is the entire point of the test. The four
    /// neighbouring `non_standard` tests all assert `formulaic - N`, which is
    /// mathematically invariant under the very transform this pins — both
    /// orderings produce a `silent` that sits exactly `voice_penalty` below
    /// their own `formulaic`, so a relative assertion agrees with the bug and
    /// with the fix. The fixture is equally deliberate: the two features have
    /// to **meet**, and `score + penalty` has to be odd-parity-different from
    /// `score`, or the two floors coincide and the test proves nothing.
    ///
    /// Cr 10 + Ig 1 + Sta 0 − Enc 0 + aura 0 = a raw Casting Score of 11,
    /// Deficient Ignem:
    ///
    /// | cell | correct `halve(11 + p)` | wrong `halve(11) + p` |
    /// |---|---|---|
    /// | `still` (−5) | `halve(6)` = **3** | `5 - 5` = 0 |
    /// | `silent` (−10) | `halve(1)` = **0** | `5 - 10` = −5 |
    /// | `silent_and_still` (−15) | `halve(-4)` = **−2** | `5 - 15` = −10 |
    #[test]
    fn non_standard_penalties_are_inside_the_deficient_halving() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 0);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 1,
            },
        ];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        )];
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert!(cell.deficient);
        // The raw Casting Score is 11; halved on its own it is 5.
        assert_eq!(cell.formulaic, 5);
        assert_eq!(cell.non_standard.voice_penalty, -10);
        assert_eq!(cell.non_standard.gesture_penalty, -5);
        assert_eq!(cell.non_standard.still, 3);
        assert_eq!(cell.non_standard.silent, 0);
        assert_eq!(cell.non_standard.silent_and_still, -2);
    }

    /// Quiet Magic cuts the no-voice penalty to −5; a second casting eliminates it
    /// (the residual clamps at 0). ArMDE:4822-4826.
    #[test]
    fn quiet_magic_reduces_then_eliminates_voice_penalty() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.quiet_magic"))];
        let totals = casting_totals(&e, &rs);
        let c = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(c.non_standard.voice_penalty, -5);
        assert_eq!(c.non_standard.silent, c.formulaic - 5);
        // Gestures are untouched by Quiet Magic.
        assert_eq!(c.non_standard.gesture_penalty, -5);

        // Taken twice → no-voice penalty eliminated altogether (clamp at 0).
        e.selections = vec![
            Selection::new(Id::new("virtue.quiet_magic")),
            Selection::new(Id::new("virtue.quiet_magic")),
        ];
        let totals = casting_totals(&e, &rs);
        let c = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(c.non_standard.voice_penalty, 0);
        assert_eq!(c.non_standard.silent, c.formulaic);
    }

    /// Subtle Magic removes the no-gesture penalty; voice is untouched.
    /// ArMDE:5073-5076.
    #[test]
    fn subtle_magic_removes_gesture_penalty() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.subtle_magic"))];
        let totals = casting_totals(&e, &rs);
        let c = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(c.non_standard.gesture_penalty, 0);
        assert_eq!(c.non_standard.still, c.formulaic);
        assert_eq!(c.non_standard.voice_penalty, -10);
    }

    /// Deft Form waives both penalties, but only for spells in that Form.
    /// ArMDE:3645-3648.
    #[test]
    fn deft_form_waives_both_penalties_in_that_form_only() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.deft_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        )];
        let totals = casting_totals(&e, &rs);
        let ignem = find_casting(&totals, "art.creo", "art.ignem");
        assert!(ignem.non_standard.deft_form);
        assert_eq!(ignem.non_standard.voice_penalty, 0);
        assert_eq!(ignem.non_standard.gesture_penalty, 0);
        assert_eq!(ignem.non_standard.silent, ignem.formulaic);
        assert_eq!(ignem.non_standard.still, ignem.formulaic);
        // A different Form is unaffected.
        let corpus = find_casting(&totals, "art.creo", "art.corpus");
        assert!(!corpus.non_standard.deft_form);
        assert_eq!(corpus.non_standard.voice_penalty, -10);
        assert_eq!(corpus.non_standard.gesture_penalty, -5);
    }

    /// Per-Form Magic Resistance = Form + 5 × Parma (ArMDE:9390-9398).
    #[test]
    fn magic_resistance_is_form_plus_five_parma() {
        let rs = ruleset();
        let mut e = magus();
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 4,
        }];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr
            .iter()
            .find(|m| m.form.as_str() == "art.ignem")
            .expect("ignem");
        // Ignem 4 + 5 × Parma 3 = 19.
        assert_eq!(ignem.total, 19);
        // Corpus 0 + 15 = 15.
        let corpus = mr.iter().find(|m| m.form.as_str() == "art.corpus").unwrap();
        assert_eq!(corpus.total, 15);
    }

    /// Flawed Parma Magica halves the **Parma contribution alone**, and only
    /// against the Form its own selection names: the Flaw's subject is "Your
    /// Parma Magica" (ArMDE:6144), and :9396 puts the Form half of the
    /// resistance on the Form scores rather than on the Parma, so a defective
    /// Parma cannot reduce it. Source: ArMDE:6142-6145, :9390, :9396, :9398.
    #[test]
    fn flawed_parma_halves_only_the_parma_contribution_against_its_own_form() {
        let rs = ruleset();
        let mut e = magus();
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.ignem"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 10,
            },
        ];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.flawed_parma"),
            BTreeMap::from([("form".to_string(), Id::new("art.ignem"))]),
        )];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // Ignem 10 + halve(5 × Parma 3) = 10 + 7 = 17.
        assert_eq!(ignem.total, 17);
        let addend = |m: &MagicResistance, label: &str| {
            m.addends
                .iter()
                .find(|a| a.label == label)
                .unwrap_or_else(|| panic!("a {label} addend"))
                .value
        };
        assert_eq!(addend(ignem, "form"), 10, "the Form score is not halved");
        assert_eq!(addend(ignem, "parma"), 7, "only the Parma is halved");
        // Every other Form is untouched: Corpus 10 + 15 = 25.
        let corpus = mr.iter().find(|m| m.form.as_str() == "art.corpus").unwrap();
        assert_eq!(corpus.total, 25);
        assert_eq!(addend(corpus, "parma"), 15);
    }

    /// The halving is on the *Parma* contribution, so it never halves a
    /// Might-being's blanket Might base. Might and Parma do not stack and the
    /// higher of the two is the base (RoP:M:1472; ArMDE:2627) — a defective
    /// Parma lowers only the Parma side of that comparison.
    /// Source: ArMDE:6142-6145.
    #[test]
    fn flawed_parma_never_halves_a_might_base() {
        let rs = ruleset();
        let mut e = magus();
        e.might = Some(MightScore {
            realm: Realm::Magic,
            score: 30,
        });
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 10,
        }];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.flawed_parma"),
            BTreeMap::from([("form".to_string(), Id::new("art.ignem"))]),
        )];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // Ignem 10 + max(halve(15), 30) = 10 + 30 = 40, on an unhalved Might base.
        assert_eq!(ignem.total, 40);
        assert!(
            ignem
                .addends
                .iter()
                .any(|a| a.label == "might" && a.value == 30)
        );
    }

    /// A supernatural being's Magic Resistance equals its Might Score, blanket
    /// across every Form; it does not stack with Parma — the higher is used
    /// (RoP:M:1472; ArMDE:2627).
    #[test]
    fn might_being_magic_resistance_is_might_score() {
        let rs = ruleset();
        let mut e = magus();
        e.might = Some(crate::types::MightScore {
            realm: crate::types::Realm::Infernal,
            score: 5,
        });
        let mr = magic_resistance(&e, &rs);
        // No Parma, no Arts: every Form's MR is the Might Score (5), via a "might"
        // addend (not "parma").
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        assert_eq!(ignem.total, 5);
        assert!(ignem.addends.iter().any(|a| a.label == "might"));
    }

    /// Might and Parma do not stack: the base uses whichever is higher (ArMDE:2627).
    #[test]
    fn might_does_not_stack_with_parma_uses_higher() {
        let rs = ruleset();
        let mut e = magus();
        e.might = Some(crate::types::MightScore {
            realm: crate::types::Realm::Magic,
            score: 30,
        });
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3, // 5×3 = 15 < 30
        }];
        let mr = magic_resistance(&e, &rs);
        let corpus = mr.iter().find(|m| m.form.as_str() == "art.corpus").unwrap();
        // max(15, 30) + Corpus 0 = 30.
        assert_eq!(corpus.total, 30);
    }

    /// A True Faith Score is a Magic Resistance **floor across every Form**:
    /// "A character with a True Faith Score gains Magic Resistance equal to this
    /// score multiplied by ten" (ArMDE:17611), and "these totals do not stack …
    /// you simply use the higher total" (ArMDE:2627) — so it competes with, not
    /// adds to, a weak Form's ordinary total (D37, F-329).
    #[test]
    fn true_faith_floors_a_weak_forms_magic_resistance() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.true_faith"))];
        // No Parma, no Arts: every Form's ordinary total is 0, below the floor.
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        assert_eq!(ignem.total, 10, "True Faith 1 x 10 = 10 floors the Form");
        assert!(
            ignem
                .addends
                .iter()
                .any(|a| a.label == "true_faith" && a.value == 10)
        );
    }

    /// The floor never suppresses a stronger ordinary total — it is a `max`, not
    /// a replacement (D37, ArMDE:2627 "the higher total").
    #[test]
    fn true_faith_does_not_lower_a_strong_forms_magic_resistance() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.true_faith"))];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 10,
        }];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // Ignem 10 + Parma 15 = 25, above the True Faith floor of 10.
        assert_eq!(ignem.total, 25);
        assert!(ignem.addends.iter().any(|a| a.label == "parma"));
    }

    /// The floor is a separate source from Parma, so it is untouched by a Flaw
    /// that reduces only the Parma/Form side — it can override that reduction
    /// on the very Form the Flaw names (D37: True Faith competes with the WHOLE
    /// per-Form total, ArMDE:2627).
    #[test]
    fn true_faith_floor_overrides_a_flawed_parma_reduction() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("virtue.true_faith")),
            Selection::new(Id::new("virtue.true_faith")),
            Selection::with_params(
                Id::new("flaw.flawed_parma"),
                BTreeMap::from([("form".to_string(), Id::new("art.ignem"))]),
            ),
        ];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.ignem"),
            score: 10,
        }];
        let mr = magic_resistance(&e, &rs);
        // Ignem: 10 + halve(15) = 17, reduced by the Flaw, but True Faith 2 x 10
        // = 20 is the higher total, so the floor wins on this Form too.
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        assert_eq!(ignem.total, 20);
        assert!(
            ignem
                .addends
                .iter()
                .any(|a| a.label == "true_faith" && a.value == 20)
        );
        // Corpus is untouched by the Flaw (0 + 15 = 15), also below the floor.
        let corpus = mr.iter().find(|m| m.form.as_str() == "art.corpus").unwrap();
        assert_eq!(corpus.total, 20);
    }

    /// Per-known-spell penetration = Casting Total − Level + Penetration score;
    /// Weak Magic halves after subtracting level (ArMDE:9159-9161, :7064-7067).
    #[test]
    fn penetration_per_spell_and_weak_magic() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.aura = 0;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.penetration"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        }];
        let pen = penetration(&e, &rs);
        assert_eq!(pen.len(), 1);
        // Casting Total = Cr10 + Ig5 + Sta2 = 17; - level 20 + Penetration 4 = 1.
        assert_eq!(pen[0].casting_total, 17);
        assert_eq!(pen[0].total, 1);

        // With Weak Magic: floor((17 - 20 + 4)/2) = 0 (ArMDE:547, rounds down).
        e.selections = vec![Selection::new(Id::new("flaw.weak_magic"))];
        let pen = penetration(&e, &rs);
        assert!(pen[0].weak_magic);
        assert_eq!(pen[0].total, 0);
    }

    /// An unclamped `Entity::aura` reaches `formulaic_casting_score` (via
    /// `penetration`'s `casting_total`) and the `pen` closure's own
    /// `casting - level + penetration_ability` sum. Both must saturate, not wrap
    /// — see the sibling casting/lab-total overflow tests for the same guard.
    #[test]
    fn an_out_of_range_aura_saturates_penetration_instead_of_overflowing() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.aura = i32::MAX; // deliberately unclamped — normalize() was not called
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.penetration"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        }];
        let pen = penetration(&e, &rs);
        assert_eq!(pen.len(), 1);
        assert_eq!(
            pen[0].casting_total,
            i32::MAX,
            "the casting total saturates rather than wraps"
        );
        assert_eq!(
            pen[0].total,
            i32::MAX - 20 + 4,
            "casting_total (saturated at i32::MAX) − level 20 + penetration 4, no overflow"
        );
    }

    /// A parameterized meta-magic Vim spell's Casting Total and Penetration use
    /// its catalogue Vim Arts (MuVi), NOT the chosen target-Form parameter, and
    /// the Penetration line carries the chosen parameter so two instances of the
    /// one spell id are distinct. Source: ArMDE:15791-15794 (the (Form) is
    /// the target spell's Form; these are MuVi spells).
    #[test]
    fn parametrized_spell_penetration_uses_vim_not_param_form() {
        let rs = ruleset();
        let mut e = magus();
        e.aura = 0;
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.muto"),
                score: 3,
            },
            ArtScore {
                art: Id::new("art.vim"),
                score: 4,
            },
            // A high target-Form score that MUST NOT feed the casting total.
            ArtScore {
                art: Id::new("art.ignem"),
                score: 20,
            },
        ];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.penetration"),
            parameter: None,
            specialty: None,
            score: 0,
        }];
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.wizards_boost_form"),
            level: Some(10),
            mastery: None,
            parameter: Some("art.ignem".into()),
            mastery_abilities: Vec::new(),
        }];
        let pen = penetration(&e, &rs);
        assert_eq!(pen.len(), 1);
        // Casting Total = Muto3 + Vim4 + Sta2 = 9 — the catalogue MuVi Arts, NOT
        // the Ignem20 target Form.
        assert_eq!(pen[0].casting_total, 9);
        // Penetration = 9 − level 10 + Penetration 0 = −1.
        assert_eq!(pen[0].total, -1);
        // The line carries the chosen parameter, distinguishing instances.
        assert_eq!(pen[0].parameter, Some("art.ignem".to_string()));
    }

    /// A grog combat line with a weapon and a shield combined (ArMDE:16658-16670,
    /// :16656).
    #[test]
    fn combat_line_combines_weapon_and_shield() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("shield.round"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
        ];
        let lines = combat_totals(&e, &rs);
        // A one-handed weapon plus a shield offers the fighter a real choice, so both
        // ways of wielding it are printed: with the shield first, then bare.
        assert_eq!(lines.len(), 2);

        let with_shield = &lines[0];
        // Total Load 2 (sword 1 + shield 1) → Burden 1; Str 3 → Encumbrance 0.
        // Init = Qik 1 + WpnInit 2 + ShieldInit 0 − Enc 0 = 3.
        assert_eq!(with_shield.initiative, 3);
        // Attack = Dex 2 + Ability 4 + WpnAtk 4 + ShieldAtk 0 = 10.
        assert_eq!(with_shield.attack, Some(10));
        // Defense = Qik 1 + Ability 4 + WpnDef 1 + ShieldDef 2 = 8.
        assert_eq!(with_shield.defense, 8);
        // Damage = Str 3 + WpnDam 6 = 9.
        assert_eq!(with_shield.damage, Some(9));
        // The line names the shield it folded in, so the renderers can label it.
        assert_eq!(with_shield.shields, vec![Id::new("shield.round")]);

        let bare = &lines[1];
        // The bare line differs ONLY by the shield's modifiers: Defense loses the +2.
        assert_eq!(bare.initiative, 3);
        assert_eq!(bare.attack, Some(10));
        assert_eq!(bare.defense, 6);
        assert_eq!(bare.damage, Some(9));
        assert!(bare.shields.is_empty());
        // The shield still weighs on Encumbrance in both lines (ArMDE:17107).
        assert_eq!(encumbrance(&e, &rs).total, 0);
    }

    /// With no shield equipped there is nothing to choose between, so the weapon
    /// yields exactly one (bare) line — byte-identical to the pre-shield behavior.
    #[test]
    fn bare_line_when_no_shield_is_equipped_is_the_only_line() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].shields.is_empty());
        // Defense = Qik 1 + Ability 4 + WpnDef 1 = 6, no shield anywhere.
        assert_eq!(lines[0].defense, 6);
    }

    /// K5 red (`docs/vf-audit/design-f0-book-template-engine.md` § 2a): a
    /// `Carried` weapon yields a Combat row but contributes no Load — the
    /// Knight's own carried great sword (ArMDE:1470-1471). The row-emission
    /// filter in `combat_totals` still reads `== Wielded` only (the F1 stub,
    /// left unchanged pending the green-phase K5 fix — see the `TODO(K5, ...)`
    /// comment there), so this fails today: a Carried weapon yields no line at
    /// all.
    #[test]
    fn a_carried_weapon_yields_a_combat_row_with_no_load() {
        let rs = ruleset();
        let mut e = grog();
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.great_sword"),
            loadout: LoadoutState::Carried,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        assert_eq!(
            lines.len(),
            1,
            "a Carried weapon must still yield a Combat row (K5)"
        );
        assert_eq!(encumbrance(&e, &rs).load, 0, "Carried gear adds no Load");
    }

    /// Several equipped shields stay summed into one pseudo-shield, and the
    /// with-shield line lists every one of them.
    #[test]
    fn multiple_shields_are_summed_and_all_listed() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        let round_shield = || EquipmentSlot {
            item: Id::new("shield.round"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        };
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            round_shield(),
            round_shield(),
        ];
        let lines = combat_totals(&e, &rs);
        assert_eq!(lines.len(), 2);
        // Load 3 (sword 1 + 2 shields) → Burden 2; Str 3 → Enc 0.
        assert_eq!(encumbrance(&e, &rs).total, 0);
        // Defense = Qik 1 + Ability 4 + WpnDef 1 + 2 × ShieldDef 2 = 10.
        assert_eq!(lines[0].defense, 10);
        assert_eq!(
            lines[0].shields,
            vec![Id::new("shield.round"), Id::new("shield.round")]
        );
        // The bare line drops all four points of shield Defense.
        assert_eq!(lines[1].defense, 6);
        assert!(lines[1].shields.is_empty());
    }

    /// Issue B: a two-handed weapon wielded with a shield gets NO shield
    /// Init/Attack/Defense modifiers, but the shield still adds to Load /
    /// Encumbrance (ArMDE:7494, :17107).
    #[test]
    fn two_handed_weapon_ignores_shield_mods() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 0); // low Str so Load matters.
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.great_sword"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("shield.round"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
        ];
        // Total Load 2 (great sword) + 1 (shield) = 3 → Burden 2; Str 0 → Enc 2.
        assert_eq!(encumbrance(&e, &rs).total, 2);
        let lines = combat_totals(&e, &rs);
        // No shield line to choose between: the shield's modifiers never apply, so
        // there is exactly one line and it names no shield.
        assert_eq!(lines.len(), 1);
        assert!(lines[0].shields.is_empty());
        let l = &lines[0];
        // Init = Qik 1 + WpnInit 2 + ShieldInit 0 − Enc 2 = 1 (shield Init 0 anyway,
        // and a two-handed weapon takes no shield Init).
        assert_eq!(l.initiative, 1);
        // Attack = Dex 2 + Ability 4 + WpnAtk 5 (+ NO ShieldAtk) = 11.
        assert_eq!(l.attack, Some(11));
        // Defense = Qik 1 + Ability 4 + WpnDef 2 (+ NO ShieldDef +2) = 7.
        assert_eq!(l.defense, 7);
        // Damage = Str 0 + WpnDam 9 = 9.
        assert_eq!(l.damage, Some(9));
    }

    /// Issue C: with the weapon's Ability carrying a specialty and the slot's
    /// `specialization_applies` toggled on, Attack and Defense gain +1; Damage and
    /// Initiative (which do not use the Ability) are unchanged (ArMDE:7122, :7139).
    #[test]
    fn specialization_adds_one_to_attack_and_defense_only() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: Some("longsword".into()),
            score: 4,
        }];
        let long_sword = || EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: true,
        };
        e.equipment = vec![long_sword()];
        let l = &combat_totals(&e, &rs)[0];
        // Attack = Dex 2 + (Ability 4 +1 spec) + WpnAtk 4 = 11.
        assert_eq!(l.attack, Some(11));
        // Defense = Qik 1 + (Ability 4 +1 spec) + WpnDef 1 = 7.
        assert_eq!(l.defense, 7);
        // Damage = Str 3 + WpnDam 6 = 9 (no Ability, so no +1).
        assert_eq!(l.damage, Some(9));
        // Init = Qik 1 + WpnInit 2 − Enc 0 = 3 (no Ability, so no +1).
        assert_eq!(l.initiative, 3);

        // Toggle OFF → no bonus.
        e.equipment = vec![EquipmentSlot {
            specialization_applies: false,
            ..long_sword()
        }];
        let off = &combat_totals(&e, &rs)[0];
        assert_eq!(off.attack, Some(10));
        assert_eq!(off.defense, 6);

        // No specialty on the Ability → no bonus even with the toggle on.
        e.ability_scores[0].specialty = None;
        e.equipment = vec![long_sword()];
        let no_spec = &combat_totals(&e, &rs)[0];
        assert_eq!(no_spec.attack, Some(10));
        assert_eq!(no_spec.defense, 6);
    }

    /// Issue A: with all Load coming from combat gear (weapons + armor), the
    /// Encumbrance penalty is exempt from Attack/Defense but still hits Initiative
    /// (ArMDE:17105, :16658).
    ///
    /// This replaces a companion test that called the private `combat_gear_is_majority`
    /// helper with hand-written `(7, 10)` / `(3, 10)` pairs. Those argument pairs
    /// were unreachable: every catalogued item that carries Load is a weapon, shield
    /// or armor, so combat Load and total Load are the same sum and the helper could
    /// only ever be asked `(n, n)`. It read as coverage while pinning a branch the
    /// engine cannot take. The assertions below go through the public
    /// `Entity` + `Ruleset` surface instead, where they pin the numbers the sheet
    /// actually shows.
    #[test]
    fn combat_gear_exempts_attack_defense_but_not_initiative() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Dex, 2);
        set_char(&mut e, Characteristic::Str, 0);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.single_weapon"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        // Weapon Load 1 + armor Load 1 = 2 → Burden 1; Str 0 → Enc 1. All combat.
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
        ];
        // The Encumbrance is real — this is not a fixture that dodges the rule by
        // carrying nothing.
        assert_eq!(encumbrance(&e, &rs).total, 1);
        let l = &combat_totals(&e, &rs)[0];
        // Init = Qik 1 + WpnInit 2 - Enc 1 = 2 (Initiative IS penalized).
        assert_eq!(l.initiative, 2);
        // Attack = Dex 2 + Ability 4 + WpnAtk 4 = 10 (NO -Enc: exempt).
        assert_eq!(l.attack, Some(10));
        // Defense = Qik 1 + Ability 4 + WpnDef 1 = 6 (NO -Enc: exempt).
        assert_eq!(l.defense, 6);
    }

    /// Soak with Tough (+3) and a Bronze cord, plus worn armor (ArMDE:16666,
    /// :5145-5147, :10840-10844).
    #[test]
    fn soak_with_tough_and_bronze_cord() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.selections = vec![Selection::new(Id::new("virtue.tough"))];
        e.familiar = Some(Familiar {
            name: "Corax".to_string(),
            cord_bronze: 2,
            ..Default::default()
        });
        e.equipment = vec![EquipmentSlot {
            item: Id::new("armor.leather_scale"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let s = soak(&e, &rs);
        // Sta 2 + Armor 3 + Tough 3 + Bronze 2 + Form 0 = 10.
        assert_eq!(s.total, 10);
    }

    /// Encumbrance from a hand-set Load: armor Load 1 → Burden 1; Str 0 → Enc 1
    /// (ArMDE:17103-17123).
    #[test]
    fn encumbrance_from_load_table() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Str, 0);
        e.equipment = vec![EquipmentSlot {
            item: Id::new("armor.leather_scale"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let enc = encumbrance(&e, &rs);
        assert_eq!(enc.load, 1);
        assert_eq!(enc.burden, 1);
        assert_eq!(enc.total, 1);
    }

    /// Only **equipped** gear counts toward Load, so a stowed spare weapon costs
    /// the character nothing.
    ///
    /// The rule says to total "the Load that a character is carrying"
    /// (ArMDE:17107) and never defines carried-but-stowed, so the book's own
    /// worked characters are the tie-breaker. The Knight template
    /// (ArMDE:1447-1486) lists four items — full chain mail, long sword, heater
    /// shield **and** a great sword — and prints "Encumbrance: 2 (3)"
    /// (ArMDE:1484). Burden 3 is Load 6-9, so the printed figure counts the
    /// wielded set only: long sword 1 + heater 2 + chain 6 = 9. Totalling all
    /// four gives 11, which is Burden 4. The reading is confirmed by his *other*
    /// loadout landing on the same Burden — great sword 2 + chain 6 = 8, also
    /// Burden 3 — which is why one printed Encumbrance serves all four of his
    /// Combat rows.
    #[test]
    fn only_equipped_gear_counts_toward_load() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Str, 0);
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                loadout: LoadoutState::Stowed,
                specialization_applies: false,
            },
        ];
        let enc = encumbrance(&e, &rs);
        assert_eq!(enc.load, 1, "the stowed second item must not add Load");
        assert_eq!(enc.burden, 1);
        assert_eq!(enc.total, 1);
    }

    // --- F2/K3: mounted combat (design-f0-book-template-engine.md § 2b, D66)
    // — RED CHECKPOINT: `combat_totals`'s mounted-twin pass is not implemented
    // yet (phase 2); every line built today has `mounted: false` unconditionally
    // (the stub), so every "twin present" lookup below panics until it lands.

    /// "A mounted character adds his Ride score, to a maximum of +3, to his
    /// Attack and Defense Totals" (ArMDE:16839). Ride 5 caps at +3; Initiative
    /// and Damage are untouched.
    #[test]
    fn mounted_twin_adds_ride_capped_at_three_to_attack_and_defense() {
        let rs = ruleset();
        let mut e = grog();
        e.mounted = true;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.ride"),
            score: 5,
            specialty: None,
            parameter: None,
        }];
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        let base = lines
            .iter()
            .find(|l| l.weapon.as_str() == "weapon.long_sword" && !l.mounted)
            .expect("unmounted line present");
        let twin = lines
            .iter()
            .find(|l| l.weapon.as_str() == "weapon.long_sword" && l.mounted)
            .expect("mounted twin present — Ride 5 caps at +3 (ArMDE:16839)");
        assert_eq!(twin.attack, base.attack.map(|a| a + 3));
        assert_eq!(twin.defense, base.defense + 3);
        assert_eq!(twin.initiative, base.initiative, "Initiative is untouched");
        assert_eq!(twin.damage, base.damage, "Damage is untouched");
    }

    /// Below the +3 cap, the Ride bonus is uncapped: Ride 2 adds +2.
    #[test]
    fn mounted_twin_ride_bonus_is_uncapped_below_three() {
        let rs = ruleset();
        let mut e = grog();
        e.mounted = true;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.ride"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        let base = lines
            .iter()
            .find(|l| l.weapon.as_str() == "weapon.long_sword" && !l.mounted)
            .expect("unmounted line present");
        let twin = lines
            .iter()
            .find(|l| l.weapon.as_str() == "weapon.long_sword" && l.mounted)
            .expect("mounted twin present at Ride 2 (+2, uncapped)");
        assert_eq!(twin.attack, base.attack.map(|a| a + 2));
        assert_eq!(twin.defense, base.defense + 2);
    }

    /// D66: the mounted twin is gated on `Weapon::body_attack` alone. Dodge
    /// (a body attack) must get no mounted twin, reproducing the Knight's own
    /// template (no mounted Fist row, ArMDE:1467-1472).
    #[test]
    fn body_attack_weapon_gets_no_mounted_twin() {
        let rs = ruleset();
        let mut e = grog();
        e.mounted = true;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.ride"),
            score: 5,
            specialty: None,
            parameter: None,
        }];
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.dodge"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        assert!(
            !lines
                .iter()
                .any(|l| l.weapon.as_str() == "weapon.dodge" && l.mounted),
            "a body attack (Weapon::body_attack) must get no mounted twin (D66)"
        );
    }

    /// Control case: `entity.mounted` defaults to `false`, so an unmounted
    /// entity's combat lines never carry a mounted twin (already true today,
    /// since the twin pass does not exist yet — pinned so a future refactor
    /// cannot flip it).
    #[test]
    fn unmounted_entity_produces_no_mounted_lines() {
        let rs = ruleset();
        let mut e = grog();
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        assert!(
            !lines.iter().any(|l| l.mounted),
            "entity.mounted defaults to false"
        );
    }

    /// Wound ranges for Size 0 and Size +1 (ArMDE:17167-17180).
    #[test]
    fn wound_ranges_scale_with_size() {
        let rs = ruleset();
        let e = grog(); // Size 0 (no SizeDelta).
        let w = wound_ranges(&e, &rs);
        let light = w.iter().find(|b| b.level == WoundBand::Light).unwrap();
        assert_eq!((light.min, light.max), (1, Some(5)));
        let medium = w.iter().find(|b| b.level == WoundBand::Medium).unwrap();
        assert_eq!((medium.min, medium.max), (6, Some(10)));
        let heavy = w.iter().find(|b| b.level == WoundBand::Heavy).unwrap();
        assert_eq!((heavy.min, heavy.max), (11, Some(15)));
        let incap = w
            .iter()
            .find(|b| b.level == WoundBand::Incapacitating)
            .unwrap();
        assert_eq!((incap.min, incap.max), (16, Some(20)));
        let dead = w.iter().find(|b| b.level == WoundBand::Dead).unwrap();
        assert_eq!((dead.min, dead.max), (21, None));
        assert_eq!(light.penalty, Some(-1));
        assert_eq!(medium.penalty, Some(-3));
    }

    /// Size +1 widens every band by the unit growth (ArMDE:17167-17180).
    #[test]
    fn wound_ranges_size_plus_one() {
        // Directly exercise the band formula at Size +1 (u = 6).
        let u = 6;
        assert_eq!((1, u), (1, 6)); // light
        assert_eq!((u + 1, 2 * u), (7, 12)); // medium
        assert_eq!((2 * u + 1, 3 * u), (13, 18)); // heavy
        assert_eq!((3 * u + 1, 4 * u), (19, 24)); // incap
        assert_eq!(4 * u + 1, 25); // dead
    }

    /// Enduring Constitution reduces wound and fatigue penalty magnitudes
    /// (ArMDE:3751-3754).
    #[test]
    fn enduring_constitution_reduces_penalties() {
        let rs = ruleset();
        let mut e = grog();
        e.selections = vec![Selection::new(Id::new("virtue.enduring_constitution"))];
        let f = fatigue_levels(&e, &rs);
        // Weary −1 + 1 = 0; Tired −3 + 1 = −2.
        assert_eq!(
            f.iter()
                .find(|l| l.level == FatigueTier::Weary)
                .unwrap()
                .penalty,
            0
        );
        assert_eq!(
            f.iter()
                .find(|l| l.level == FatigueTier::Tired)
                .unwrap()
                .penalty,
            -2
        );
        let w = wound_ranges(&e, &rs);
        // Light −1 + 1 = 0; Medium −3 + 1 = −2.
        assert_eq!(
            w.iter()
                .find(|b| b.level == WoundBand::Light)
                .unwrap()
                .penalty,
            Some(0)
        );
        assert_eq!(
            w.iter()
                .find(|b| b.level == WoundBand::Medium)
                .unwrap()
                .penalty,
            Some(-2)
        );
    }

    /// Decrepitude (17 aging points → 2) and Warping (15 points → 2) are reused
    /// from `effective.rs`, not reimplemented (ArMDE:16617, :16464-16475).
    #[test]
    fn decrepitude_and_warping_reuse_effective() {
        let rs = ruleset();
        let mut e = magus();
        e.aging_points.insert(Characteristic::Str, 10);
        e.aging_points.insert(Characteristic::Sta, 7); // 17 total → Decrepitude 2.
        e.warping_points = 15; // → Warping 2.
        let d = derived_totals(&e, &rs);
        assert_eq!(d.decrepitude_score, 2);
        assert_eq!(d.warping_score, 2);
        assert_eq!(d.warping_points, 15);
    }

    /// A grog gets no magic totals; a magus does.
    #[test]
    fn magic_totals_only_for_magi() {
        let rs = ruleset();
        let g = derived_totals(&grog(), &rs);
        assert!(!g.hermetically_trained);
        assert!(g.lab_totals.is_empty());
        assert!(g.casting_totals.is_empty());
        let m = derived_totals(&magus(), &rs);
        assert!(m.hermetically_trained);
        assert!(!m.lab_totals.is_empty());
        assert!(!m.casting_totals.is_empty());
    }

    /// Surfaced-only modifiers (Apt Student) are listed, not folded into a number.
    /// D45/F-423: the row also names its source, so two carriers of the same
    /// family+detail read as two distinguishable lines rather than one
    /// anonymous word repeated.
    #[test]
    fn surfaced_modifiers_are_listed() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.apt_student"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "taught"
            && m.amount == 5
            && m.source == Some(Id::new("virtue.apt_student"))));
    }

    /// Good Teacher's authoring bonus (E2/Q-32) surfaces through the new
    /// `AdvancementSource::Authoring`, distinct from its own teaching row.
    #[test]
    fn surfaced_modifiers_include_the_authoring_source() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.good_teacher"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "teaching"
            && m.amount == 5));
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "authoring"
            && m.amount == 3));
    }

    /// D55 (Q6, V/F audit Q-113): Incomprehensible halves along BOTH axes —
    /// teaching and authoring — and Loose Magic halves Spell Mastery. All
    /// three surface with `factor: Some(Half)`, never an `amount`.
    #[test]
    fn surfaced_modifiers_carry_a_halving_factor_not_a_zero_amount() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("flaw.incomprehensible")),
            Selection::new(Id::new("flaw.loose_magic")),
        ];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "teaching"
            && m.factor == Some(AdvancementFactor::Half)));
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "authoring"
            && m.factor == Some(AdvancementFactor::Half)));
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "spell_mastery"
            && m.factor == Some(AdvancementFactor::Half)));
    }

    /// Inventive Genius folds a flat +3 into the Lab-Total `lab_mod` addend of
    /// every cell (ArMDE:4151-4154).
    #[test]
    fn lab_total_mod_adds_to_every_cell() {
        let rs = ruleset();
        let mut e = magus();
        // All scores 0, aura 0, so the only contribution is the lab_mod.
        e.selections = vec![Selection::new(Id::new("virtue.inventive_genius"))];
        let totals = lab_totals(&e, &rs);
        let cell = totals
            .iter()
            .find(|t| t.technique.as_str() == "art.creo" && t.form.as_str() == "art.corpus")
            .expect("lab cell present");
        let lab_mod = cell.addends.iter().find(|a| a.label == "lab_mod").unwrap();
        assert_eq!(lab_mod.value, 3);
        assert_eq!(cell.total, 3);
    }

    /// Every `lab_total_mod` carrier in the SHIPPED catalogue must be
    /// classified by [`in_play_lab_total_mod`]'s D4 fold membership — either
    /// hard-excluded, cycle-gated (`flaw.cyclic_magic_negative` only), or left
    /// flat/included. There is no exhaustive-match safeguard for a per-entry
    /// distinction (unlike `Effect`'s variants), so this enumeration stands in
    /// for one: a tenth carrier added later must be triaged here rather than
    /// silently defaulting to "included" (X7a, `tmp/x7a-handover.md`).
    #[test]
    fn lab_total_mod_carriers_match_the_d4_table() {
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: include_str!("../../../rules/core/virtues_flaws.json"),
            type_profiles: include_str!("../../../rules/core/character_types.json"),
            abilities: Some(include_str!("../../../rules/core/abilities.json")),
            arts: Some(include_str!("../../../rules/core/arts.json")),
            houses: Some(include_str!("../../../rules/core/houses.json")),
            mythic_types: Some(include_str!(
                "../../../rules/core/mythic_companion_types.json"
            )),
            spells: Some(include_str!("../../../rules/core/spells.json")),
            spell_mastery_abilities: None,
            equipment: Some(include_str!("../../../rules/core/equipment.json")),
            characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
            life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
            childhoods: None,
            aging: Some(include_str!("../../../rules/core/aging.json")),
            parameter_catalogues: Some(include_str!(
                "../../../rules/core/parameter_catalogues.json"
            )),
        })
        .expect("shipped core ruleset loads");

        let carriers: BTreeSet<&str> = rs
            .point_items
            .iter()
            .filter(|(_, item)| {
                item.effects
                    .iter()
                    .any(|e| matches!(e, Effect::LabTotalMod { .. }))
            })
            .map(|(id, _)| id.as_str())
            .collect();

        // `virtue.aristotelian_training` is deliberately absent: its
        // `lab_total_mod` effect is deleted at the source (X7a item 5), not
        // excluded by this fold, so it must carry no `LabTotalMod` effect at
        // all any more.
        let expected: BTreeSet<&str> = [
            "virtue.adept_laboratory_student",
            "flaw.weak_scholar",
            "virtue.cyclic_magic_positive",
            "flaw.cyclic_magic_negative",
            "flaw.creative_block",
            "virtue.inventive_genius",
            "virtue.potent_magic_major",
            "virtue.potent_magic_minor",
        ]
        .into_iter()
        .collect();
        assert_eq!(
            carriers, expected,
            "a lab_total_mod carrier was added or removed from the catalogue without updating \
             this D4 enumeration (and, if added, without triaging its fold membership)"
        );

        for &excluded in D4_EXCLUDED_FROM_LAB_GRID {
            assert!(
                carriers.contains(excluded),
                "{excluded} is in the D4 exclusion list but no longer carries lab_total_mod"
            );
        }

        // Cyclic Magic's Flaw is cycle-gated, not flatly excluded or included —
        // its behavior is asserted end-to-end by
        // `x7a_lab_rows.rs::cyclic_magic_negative_lab_penalty_is_cycle_gated`.
        assert!(!D4_EXCLUDED_FROM_LAB_GRID.contains(&CYCLIC_MAGIC_NEGATIVE));

        // Everything else in the shipped catalogue applies flat, per the
        // character-generation-default reading (Inventive Genius, Creative
        // Block).
        for &flat in &["flaw.creative_block", "virtue.inventive_genius"] {
            assert!(
                !D4_EXCLUDED_FROM_LAB_GRID.contains(&flat)
                    && !D4_WITHIN_FOCUS_ONLY.contains(&flat)
                    && flat != CYCLIC_MAGIC_NEGATIVE,
                "{flat} must apply flat in the D4 fold, not be excluded, \
                 within-focus-only, or cycle-gated"
            );
        }

        // Both Potent Magic entries are within-focus-only (X7b-d): their
        // +6/+3 must never reach the flat `in_play_lab_total_mod` fold.
        for &within_focus_only in D4_WITHIN_FOCUS_ONLY {
            assert!(
                carriers.contains(within_focus_only),
                "{within_focus_only} is in the D4 within-focus-only list but \
                 no longer carries lab_total_mod"
            );
            assert!(
                !D4_EXCLUDED_FROM_LAB_GRID.contains(&within_focus_only),
                "{within_focus_only} cannot be both within-focus-only and hard-excluded"
            );
        }
    }

    /// Weak Enchanter (GD3, round 2): "Halve your Lab Total whenever you
    /// create or investigate an enchanted item. If you have a Deficiency that
    /// counts as part of the Lab Total, apply the Deficiency first and then
    /// halve the remaining total" (ArMDE:7060-7063). `HalvableTotal::LabEnchanting` was asserted by
    /// ruleset data and folded into `InPlayMods.halvings` but nothing ever
    /// consumed it, so the Flaw had no mechanical effect at all. `LabTotal`
    /// gains an `enchanting` field: the ordinary (Deficient-halved) `total`,
    /// halved again for a Weak Enchanter — the specified order.
    #[test]
    fn weak_enchanter_halves_the_lab_total_for_enchanting_only() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("flaw.weak_enchanter"))];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 6,
            },
        ];
        let totals = lab_totals(&e, &rs);
        let cell = totals
            .iter()
            .find(|t| t.technique.as_str() == "art.creo" && t.form.as_str() == "art.corpus")
            .expect("lab cell present");
        // total = Cr10 + Co6 = 16, not deficient; enchanting = halve(16) = 8.
        assert_eq!(cell.total, 16);
        assert_eq!(cell.enchanting, 8);
    }

    /// The Deficiency applies first, then Weak Enchanter halves what remains
    /// — composing rather than being independent of each other.
    #[test]
    fn weak_enchanter_and_deficient_art_compose_deficiency_first() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("flaw.weak_enchanter")),
            Selection::with_params(
                Id::new("flaw.deficient_technique"),
                BTreeMap::from([("art".into(), Id::new("art.creo"))]),
            ),
        ];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 6,
            },
        ];
        let totals = lab_totals(&e, &rs);
        let cell = totals
            .iter()
            .find(|t| t.technique.as_str() == "art.creo" && t.form.as_str() == "art.corpus")
            .expect("lab cell present");
        assert!(cell.deficient);
        // Deficient first: halve(16) = 8; Weak Enchanter then halves that: 4.
        assert_eq!(cell.total, 8);
        assert_eq!(cell.enchanting, 4);
    }

    /// A cell the Flaw does not touch (no Weak Enchanter selected) reports the
    /// ordinary total unchanged at both fields.
    #[test]
    fn enchanting_equals_total_without_weak_enchanter() {
        let rs = ruleset();
        let mut e = magus();
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 6,
            },
        ];
        let totals = lab_totals(&e, &rs);
        let cell = totals
            .iter()
            .find(|t| t.technique.as_str() == "art.creo" && t.form.as_str() == "art.corpus")
            .expect("lab cell present");
        assert_eq!(cell.enchanting, cell.total);
    }

    /// A CombatMod (Lame, −3 Initiative) folds into the Initiative combat total.
    #[test]
    fn combat_mod_adjusts_initiative() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Str, 3); // Str 3 → Encumbrance 0 with Load 1.
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            loadout: LoadoutState::Wielded,
            specialization_applies: false,
        }];
        // Baseline Init = Qik 1 + WpnInit 2 − Enc 0 = 3.
        assert_eq!(combat_totals(&e, &rs)[0].initiative, 3);
        e.selections = vec![Selection::new(Id::new("flaw.lame"))];
        // With Lame (−3 Initiative): 3 − 3 = 0.
        assert_eq!(combat_totals(&e, &rs)[0].initiative, 0);
    }

    /// A weapon-scoped CombatMod replaces the same item's unscoped figure on that
    /// weapon's line alone. Lame reads "-3 on Dodge, and -1 on other combat
    /// scores" (ArMDE:6332): the Dodge line takes -3, not -1 and not -4, while
    /// every other weapon's line takes -1.
    #[test]
    fn weapon_scoped_combat_mod_replaces_general_on_that_weapon_only() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Str, 3);
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("weapon.dodge"),
                loadout: LoadoutState::Wielded,
                specialization_applies: false,
            },
        ];
        let line = |lines: &[CombatLine], id: &str| {
            lines
                .iter()
                .find(|l| l.weapon.as_str() == id)
                .expect("line present")
                .defense
        };
        // Baselines: sword = Qik 1 + WpnDef 1 = 2; dodge = Qik 1 + WpnDef 0 = 1.
        let base = combat_totals(&e, &rs);
        assert_eq!(line(&base, "weapon.long_sword"), 2);
        assert_eq!(line(&base, "weapon.dodge"), 1);

        e.selections = vec![Selection::new(Id::new("flaw.lame_split"))];
        let lamed = combat_totals(&e, &rs);
        // The unscoped -1 reaches the sword; the Dodge-scoped -3 replaces it on Dodge.
        assert_eq!(line(&lamed, "weapon.long_sword"), 1);
        assert_eq!(line(&lamed, "weapon.dodge"), -2);
    }

    /// Limited Magic Resistance (a MagicResistanceMod NoFormBonus) drops the Form
    /// contribution of the **one** Form its selection names — "You gain no bonus
    /// from *one of* your Form scores" (ArMDE:6348) — leaving resistance from
    /// Parma alone against that Form and every other Form untouched.
    /// Source: ArMDE:6346-6349.
    #[test]
    fn limited_magic_resistance_drops_the_form_bonus_of_its_own_form_only() {
        let rs = ruleset();
        let mut e = magus();
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.parma_magica"),
            parameter: None,
            specialty: None,
            score: 3,
        }];
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.ignem"),
                score: 4,
            },
            ArtScore {
                art: Id::new("art.corpus"),
                score: 4,
            },
        ];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.limited_magic_resistance"),
            BTreeMap::from([("form".to_string(), Id::new("art.ignem"))]),
        )];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // Form bonus dropped: 0 + 5 × Parma 3 = 15 (not 19).
        assert_eq!(ignem.total, 15);
        let form_addend = ignem.addends.iter().find(|a| a.label == "form").unwrap();
        assert_eq!(form_addend.value, 0);
        // The Form nobody named keeps its bonus: Corpus 4 + 15 = 19.
        let corpus = mr.iter().find(|m| m.form.as_str() == "art.corpus").unwrap();
        assert_eq!(corpus.total, 19);
    }

    /// An AgingMod (Unaging → no_aging) is surfaced with amount 0, not simulated.
    /// D45: names its source (`virtue.unaging`).
    #[test]
    fn aging_mod_is_surfaced() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.unaging"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::Aging
            && m.detail == "no_aging"
            && m.amount == 0
            && m.source == Some(Id::new("virtue.unaging"))));
    }

    /// The multi-variant SpecialCasting arm (Diedne + Life Boost) is surfaced
    /// labelled, one entry per variant, amount 0. D45: each names its own
    /// source, so the two rows (both `amount: 0`, both `SpecialCasting`) stay
    /// distinguishable from each other.
    #[test]
    fn special_casting_cluster_is_surfaced() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![
            Selection::new(Id::new("virtue.diedne_magic")),
            Selection::new(Id::new("virtue.life_boost")),
        ];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::SpecialCasting
            && m.detail == "diedne"
            && m.amount == 0
            && m.source == Some(Id::new("virtue.diedne_magic"))));
        assert!(s.iter().any(|m| m.family == ModifierFamily::SpecialCasting
            && m.detail == "life_boost"
            && m.amount == 0
            && m.source == Some(Id::new("virtue.life_boost"))));
    }

    /// A non-flat Magic-Resistance modifier (Susceptibility to Faerie power) is
    /// surfaced labelled with amount 0, not silently dropped. Only NoFormBonus is
    /// folded into the flat per-Form MR number; the realm-conditional variants are
    /// listed, because "against faerie effects" is a scope the flat figure cannot
    /// carry. Source: ArMDE:6819-6826.
    /// D45: also names its source (`flaw.susceptibility_to_faerie_power`).
    #[test]
    fn susceptibility_magic_resistance_is_surfaced() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new(
            "flaw.susceptibility_to_faerie_power",
        ))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::MagicResistance
            && m.detail == "susceptible_faerie"
            && m.amount == 0
            && m.source == Some(Id::new("flaw.susceptibility_to_faerie_power"))));
    }

    /// An AbilityRollModParam (Academic Concentration) is surfaced with the free-text
    /// subject as its detail and the bonus as its amount (ArMDE:3362-3367). D45:
    /// also names its source, so two Academic Concentrations on different
    /// subjects each say which one they are.
    #[test]
    fn ability_roll_mod_is_surfaced_with_subject() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.academic_concentration"),
            BTreeMap::from([("subject".into(), Id::new("theology"))]),
        )];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.detail == "theology"
            && m.amount == 3
            && m.source == Some(Id::new("virtue.academic_concentration"))));
    }

    /// `Effect::AbilityRollModParam` (and `Effect::MagicalFocus`) carry a `text`-domain
    /// parameter, so its value reaches the sheet verbatim as a surfaced modifier's
    /// detail. Row 10: a padded descriptor is the same descriptor, and the load-time
    /// trim is what makes that true here — so a save holding " Theology " surfaces
    /// exactly the same row as one holding "Theology", instead of a detail with
    /// stray whitespace baked into the display.
    #[test]
    fn a_padded_ability_roll_mod_subject_surfaces_trimmed() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::with_params(
            Id::new("virtue.academic_concentration"),
            BTreeMap::from([("subject".into(), Id::new(" \tTheology  "))]),
        )];
        // Through the real load door, because that is where the trim lives.
        let loaded = crate::load_entity_migrating(
            &serde_json::to_string(&e).unwrap(),
            crate::DEFAULT_SAGA_YEAR,
            &rs,
            &BTreeMap::new(),
        )
        .unwrap()
        .entity;
        let s = surfaced_modifiers(&loaded, &rs);
        assert!(
            s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
                // Trimmed, and NOT case-folded: capitalisation is the player's.
                && m.detail == "Theology"
                && m.amount == 3),
            "the padded subject must surface trimmed: {:?}",
            s.iter()
                .filter(|m| m.family == ModifierFamily::AbilityRoll)
                .collect::<Vec<_>>()
        );
    }

    /// Weak Spontaneous Magic (GD2, round 2): "You may not exert yourself when
    /// casting spontaneous magic, so you always divide your Casting Score by
    /// five" (ArMDE:7084-7086, not
    /// `ArMDE:7060-7063` — that range is Weak Enchanter, a different Flaw
    /// entirely). The Flaw does not add a second halving on top of the normal
    /// ÷2 fatiguing rate (that would silently produce ÷4, a rate that appears
    /// nowhere in the rules) — it removes the fatiguing (exert-yourself)
    /// option outright, leaving only the ÷5 rate. The formulaic total is
    /// untouched either way.
    #[test]
    fn weak_spontaneous_magic_locks_both_spontaneous_figures_to_the_divide_by_five_rate() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.selections = vec![Selection::new(Id::new("flaw.weak_spontaneous"))];
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        // base = Cr10 + Ig5 + Sta2 = 17; the only rate available is ÷5 = 3,
        // reported at both the fatiguing and non-fatiguing slots since the
        // Flaw leaves no other option to report.
        assert_eq!(cell.spontaneous_fatiguing, 3);
        assert_eq!(cell.spontaneous_non_fatiguing, 3);
        // Formulaic is not a spontaneous total and is not halved.
        assert_eq!(cell.formulaic, 17);
    }

    /// Deficient Art still halves the base once before the Weak-Spontaneous
    /// ÷5 rate applies (the two Flaws address different totals — Deficient
    /// halves the underlying score, Weak Spontaneous fixes the *divisor* —
    /// so they compose rather than double-halve).
    #[test]
    fn deficient_art_and_weak_spontaneous_combine() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.selections = vec![
            Selection::with_params(
                Id::new("flaw.deficient_technique"),
                BTreeMap::from([("art".into(), Id::new("art.creo"))]),
            ),
            Selection::new(Id::new("flaw.weak_spontaneous")),
        ];
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert!(cell.deficient);
        // Formulaic: Deficient only → halve(17) = 8.
        assert_eq!(cell.formulaic, 8);
        // Spontaneous base = halve(17) = 8 (Deficient); ÷5 = 1, reported at
        // both slots (Weak Spontaneous removes the fatiguing option).
        assert_eq!(cell.spontaneous_fatiguing, 1);
        assert_eq!(cell.spontaneous_non_fatiguing, 1);
    }

    /// The per-spell penetration path halves the casting score for a Deficient Art
    /// (`formulaic_casting_score`, ArMDE:5913-5915).
    #[test]
    fn deficient_art_halves_penetration_casting_score() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 10,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 5,
            },
        ];
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.penetration"),
            parameter: None,
            specialty: None,
            score: 4,
        }];
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        }];
        e.selections = vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("art".into(), Id::new("art.creo"))]),
        )];
        let pen = penetration(&e, &rs);
        // Casting score = halve(Cr10 + Ig5 + Sta2) = halve(17) = 8 (Deficient).
        assert_eq!(pen[0].casting_total, 8);
        // Penetration = 8 − level 20 + Penetration 4 = −8.
        assert_eq!(pen[0].total, -8);
    }

    /// Every halving in the engine divides without the rule naming a direction, so
    /// the rulebook default applies: "if it does not, round down" (ArMDE:547).
    /// Floor and truncate-toward-zero agree on non-negative operands — which is why
    /// every worked example passes either way — and diverge on negative ones, which
    /// are ordinary play (a Dominion aura, a low-Art magus, Deficient Technique).
    #[test]
    fn halving_rounds_down_not_toward_zero() {
        // Negative, odd: floor(-7/2) = -4, not -3.
        assert_eq!(halve(-7), -4);
        assert_eq!(halve(-5), -3);
        assert_eq!(halve(-1), -1);
        // Negative, even: exact either way.
        assert_eq!(halve(-4), -2);
        // Non-negative: unchanged.
        assert_eq!(halve(0), 0);
        assert_eq!(halve(17), 8);
    }

    /// The same rounding default, end-to-end through `casting_totals` on a cell
    /// driven negative by a Dominion aura (ArMDE:547). A newly gauntleted magus
    /// with no Arts, Stamina −1 and no equipment: the whole grid is negative.
    #[test]
    fn a_negative_casting_cell_rounds_its_divisions_down() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, -1);

        // Casting Score = Cr0 + Ig0 + Sta−1 + Enc0 + Aura−3 = −4.
        e.aura = -3;
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(cell.formulaic, -4);
        // ÷5 floors: −4/5 = −1, not 0.
        assert_eq!(cell.spontaneous_non_fatiguing, -1);
        // ÷2 is exact here.
        assert_eq!(cell.spontaneous_fatiguing, -2);

        // Casting Score = −5: now the ÷2 is the one that rounds.
        e.aura = -4;
        let totals = casting_totals(&e, &rs);
        let cell = find_casting(&totals, "art.creo", "art.ignem");
        assert_eq!(cell.formulaic, -5);
        // ÷2 floors: −5/2 = −3, not −2.
        assert_eq!(cell.spontaneous_fatiguing, -3);
        assert_eq!(cell.spontaneous_non_fatiguing, -1);
    }

    /// A surfaced-only health-roll track (Long-Winded → fatigue_roll +3) is listed
    /// under family "health_roll", and does not perturb the folded Fatigue
    /// penalties (ArMDE:17127-17129).
    ///
    /// D45 deliberately does NOT extend to `HealthRoll`: unlike the other five
    /// families, which push one row per producing selection directly inside
    /// `in_play_mods`, a surfaced health track is read back out of
    /// `InPlayMods::health_mods`, which already sums every contributing
    /// selection's amount into one number before `surfaced_modifiers` builds
    /// this row — there is no single item left to name (Q-81's twelve-row
    /// census found more than one Virtue/Flaw commonly sharing a track). So
    /// `source` stays `None` here, pinned by the assertion below.
    #[test]
    fn health_roll_track_is_surfaced_not_folded() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.long_winded"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::HealthRoll
            && m.detail == "fatigue_roll"
            && m.amount == 3
            && m.source.is_none()));
        // The fatigue-penalty track is untouched: Weary stays −1.
        let f = fatigue_levels(&e, &rs);
        assert_eq!(
            f.iter()
                .find(|l| l.level == FatigueTier::Weary)
                .unwrap()
                .penalty,
            -1
        );
    }

    /// D45/F-423: `SurfacedModifier::source` must actually cross the IPC
    /// boundary as JSON — the frontend has nothing else to read the
    /// attribution from. `skip_serializing_if` must drop the key outright for
    /// `None` (the pre-D45 `HealthRoll` shape) rather than emit a literal
    /// `null`, since the Svelte component's `{#if m.source}` guard treats an
    /// absent key and a JS `undefined` identically but would not treat a JSON
    /// `null` that way without an explicit check.
    #[test]
    fn surfaced_modifier_source_serializes_present_or_absent() {
        let named = SurfacedModifier {
            family: ModifierFamily::Aging,
            detail: "no_aging".to_string(),
            amount: 0,
            factor: None,
            source: Some(Id::new("virtue.unaging")),
            ability: None,
        };
        let json = serde_json::to_value(&named).unwrap();
        assert_eq!(json["source"], "virtue.unaging");

        let unattributed = SurfacedModifier {
            family: ModifierFamily::HealthRoll,
            detail: "fatigue_roll".to_string(),
            amount: 3,
            factor: None,
            source: None,
            ability: None,
        };
        let json = serde_json::to_value(&unattributed).unwrap();
        assert!(
            json.get("source").is_none(),
            "a None source must be OMITTED, not serialized as null: {json:?}"
        );
    }

    /// Purity: calling `derived_totals` twice yields identical results and does not
    /// mutate the entity.
    #[test]
    fn derived_totals_is_pure() {
        let rs = ruleset();
        let mut e = magus();
        set_char(&mut e, Characteristic::Sta, 2);
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }];
        e.selections = vec![Selection::new(Id::new("virtue.method_caster"))];
        let before = e.clone();
        let a = derived_totals(&e, &rs);
        let b = derived_totals(&e, &rs);
        assert_eq!(a, b);
        assert_eq!(e, before);
    }
}
