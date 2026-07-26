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
//! Surfaced-only 5b families (study / aging-roll / non-standard-casting /
//! wound-recovery) are **listed** as labelled [`SurfacedModifier`]s rather than
//! folded into a simulated number, because the app does not simulate those
//! subsystems.
//!
//! Source line ranges (all `Ars Magica - Definitive Edition (Core Rules).md`) are
//! cited at each computing function and in `crates/arm-rules/RULES.md` (§5i).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::art::ArtType;
use crate::characteristics::Characteristic;
use crate::effective::{
    decrepitude_score, effective_ability_score, effective_art_score,
    effective_characteristic_after_aging, resolved_spell_level, selections_for_effects,
    warping_points_total, warping_score,
};
use crate::ruleset::{
    ID_ARTES_LIBERALES, ID_CORPUS, ID_CREO, ID_MAGIC_THEORY, ID_PARMA_MAGICA, ID_PENETRATION,
    ID_PHILOSOPHIAE, Ruleset,
};
use crate::types::{
    CastingScope, CombatStat, Effect, Entity, HalvableTotal, HealthTrack, Id, LongevitySource,
    MagicResistanceEffect, SpecialCasting,
};

// --- Non-standard-casting penalty constants (Core:9243-9245) ---------------

/// Casting-Score penalty for casting with **no voice** at all (the "None" Words
/// row). Source: Core:9245.
const NO_VOICE_PENALTY: i32 = -10;
/// Casting-Score penalty for casting with **no gestures** at all (the "None"
/// Gestures row). Source: Core:9245.
const NO_GESTURE_PENALTY: i32 = -5;
/// The no-voice-penalty reduction one casting of Quiet Magic grants (soft voice →
/// no penalty, no voice → −5, i.e. +5; a second casting eliminates it). Source:
/// Core:4822-4826.
const QUIET_MAGIC_VOICE_REDUCTION: i32 = 5;
/// The no-gesture-penalty reduction Subtle Magic grants (no gestures → no
/// penalty, i.e. +5). Source: Core:5073-5076.
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
    addends.iter().map(|a| a.value).sum()
}

/// Integer halving toward zero (Deficient Art / halving flaws halve *totals*).
fn halve(x: i32) -> i32 {
    x / 2
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
    /// The deficient Technique/Form Art ids (Deficient Art halves totals adding one).
    deficient_arts: BTreeSet<Id>,
    /// Whole-total halvings in effect (Weak Magic → Penetration, Flawed Parma → MR).
    halvings: BTreeSet<HalvableTotal>,
    /// Flat Soak modifier (Tough +3, Frail −1, summed).
    soak_mod: i32,
    /// Flat combat-total modifiers per stat (summed).
    combat_mods: BTreeMap<CombatStat, i32>,
    /// Health-track penalty deltas per track (positive reduces the penalty).
    health_mods: BTreeMap<HealthTrack, i32>,
    /// Non-halving Magic-Resistance modifiers (Limited MR, Susceptibility, …).
    mr_mods: Vec<MagicResistanceEffect>,
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

/// Folds all in-play (5b) effects off the entity's selections and grants. The
/// `match` is exhaustive: creation-effect and elemental variants are explicit
/// no-ops (consumed by `effective.rs`), so adding an [`Effect`] variant is a
/// compile error until it is classified here.
fn in_play_mods(entity: &Entity, ruleset: &Ruleset) -> InPlayMods {
    let mut m = InPlayMods::default();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            match effect {
                Effect::MagicalFocus { .. } => m.has_focus = true,
                Effect::MasterpieceItem => m.has_masterpiece = true,
                Effect::CastingTotalMod { amount, scope } => {
                    m.casting_mods.push((i32::from(*amount), *scope));
                }
                Effect::LabTotalMod { amount } => m.lab_mod += i32::from(*amount),
                Effect::DeficientArt { param } => {
                    if let Some(art) = selection.params.get(param) {
                        m.deficient_arts.insert(art.clone());
                    }
                }
                Effect::MagicTotalHalving { total } => {
                    m.halvings.insert(*total);
                }
                Effect::SoakMod { amount } => m.soak_mod += i32::from(*amount),
                Effect::CombatMod { amount, target } => {
                    *m.combat_mods.entry(*target).or_default() += i32::from(*amount);
                }
                Effect::HealthMod { track, amount } => {
                    *m.health_mods.entry(*track).or_default() += i32::from(*amount);
                }
                // Only NoFormBonus folds into the flat per-Form MR number
                // (magic_resistance()). The realm-conditional / situational variants
                // (aura bonus, realm susceptibilities) cannot be folded into that flat
                // figure, so they are surfaced labelled rather than silently dropped.
                // Source: Ars Magica - Definitive Edition (Core Rules).md:6815-6826
                // (Susceptibility flaws), :3579-3596 (Commanding Aura) & :4998-5001
                // (Special Circumstances) for aura_bonus.
                Effect::MagicResistanceMod { kind } => match kind {
                    MagicResistanceEffect::NoFormBonus => m.mr_mods.push(*kind),
                    MagicResistanceEffect::AuraBonus
                    | MagicResistanceEffect::SusceptibleDivine
                    | MagicResistanceEffect::SusceptibleFaerie
                    | MagicResistanceEffect::SusceptibleInfernal => {
                        m.surfaced.push(SurfacedModifier {
                            family: ModifierFamily::MagicResistance,
                            detail: kind.to_string(),
                            amount: 0,
                        })
                    }
                },
                // Surfaced-only families: listed labelled, never simulated.
                Effect::AgingMod { kind, amount } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::Aging,
                    detail: kind.to_string(),
                    amount: i32::from(*amount),
                }),
                Effect::AdvancementMod { source, amount } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::Advancement,
                    detail: source.to_string(),
                    amount: i32::from(*amount),
                }),
                // Non-standard-casting relievers are computed into the per-cell
                // NonStandardCasting variants; every other quirk stays surfaced.
                Effect::SpecialCastingMod { kind, param } => match kind {
                    SpecialCasting::QuietWords => m.voice_reduction += QUIET_MAGIC_VOICE_REDUCTION,
                    SpecialCasting::SubtleGestures => {
                        m.gesture_reduction += SUBTLE_MAGIC_GESTURE_REDUCTION
                    }
                    SpecialCasting::DeftForm => {
                        if let Some(form) = param.as_ref().and_then(|p| selection.params.get(p)) {
                            m.deft_forms.insert(form.clone());
                        }
                    }
                    SpecialCasting::Diedne
                    | SpecialCasting::FaerieRaised
                    | SpecialCasting::LifeLinkedSpontaneous
                    | SpecialCasting::SpellImprovisation
                    | SpecialCasting::Mercurian
                    | SpecialCasting::LifeBoost
                    | SpecialCasting::Circumstantial => m.surfaced.push(SurfacedModifier {
                        family: ModifierFamily::SpecialCasting,
                        detail: kind.to_string(),
                        amount: 0,
                    }),
                },
                Effect::AbilityRollMod { param, amount } => m.surfaced.push(SurfacedModifier {
                    family: ModifierFamily::AbilityRoll,
                    detail: selection
                        .params
                        .get(param)
                        .map(|id| id.as_str().to_string())
                        .unwrap_or_default(),
                    amount: i32::from(*amount),
                }),
                // Creation-effect variants (consumed by effective.rs) and the
                // Elemental Magic XP-space marker: no in-play modifier here.
                Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::GroupAffinityCost { .. }
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
                | Effect::TrueFaithGrant { .. }
                | Effect::WarpingGrant { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::GrantsReputation { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                | Effect::ElementalMagic { .. } => {}
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
    /// clamped so a Virtue can never turn it into a bonus. Source: Core:9245,
    /// :4822-4826, :3645-3648.
    fn residual_voice_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_VOICE_PENALTY + self.voice_reduction).min(0)
    }

    /// The residual Casting-Score penalty for casting a `form` spell with no
    /// gestures: Deft Form waives it, else the −5 base plus Subtle Magic reduction,
    /// clamped at 0. Source: Core:9245, :5073-5076, :3645-3648.
    fn residual_gesture_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_GESTURE_PENALTY + self.gesture_reduction).min(0)
    }
}

/// The four ways a casting total is derived (Core:9095-9145). One breakdown yields
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

// --- Lab totals (per Technique × Form) -------------------------------------

/// A Lab Total for one `(Technique, Form)` cell of the 5×10 grid.
///
/// Lab Total = Int + Magic Theory + Technique + Form + Aura + flat LabTotalMod;
/// within a Magical Focus the lower applicable Art is added again; a Deficient Art
/// halves the whole cell. Source: Core:10276-10278 (Lab Total shape), :4151-4154
/// (Inventive Genius, the flat `LabTotalMod`), :4399-4422 (focus doubling),
/// :5909-5915 (Deficient halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabTotal {
    /// Technique Art id of this cell.
    pub technique: Id,
    /// Form Art id of this cell.
    pub form: Id,
    /// The labelled base breakdown (int, magic_theory, technique, form, aura, lab_mod).
    pub addends: Vec<Addend>,
    /// The base Lab Total (after any Deficient-Art halving), no focus.
    pub total: i32,
    /// The within-focus Lab Total (base + lower Art, then halving); `None` when the
    /// magus holds no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<i32>,
    /// Whether a Deficient Art halved this cell.
    pub deficient: bool,
}

/// Lab Totals for every `(Technique, Form)` pair — the 5×10 grid. Source:
/// Core:10276-10278.
pub fn lab_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<LabTotal> {
    let mods = in_play_mods(entity, ruleset);
    let intelligence = characteristic(entity, ruleset, Characteristic::Int);
    let magic_theory = ability(entity, ruleset, ID_MAGIC_THEORY);
    let aura = entity.aura;
    let mut out = Vec::new();
    for technique in ruleset.art_ids_of(ArtType::Technique) {
        let te = effective_art_score(entity, ruleset, &technique);
        for form in ruleset.art_ids_of(ArtType::Form) {
            let fo = effective_art_score(entity, ruleset, &form);
            let addends = vec![
                Addend::new("intelligence", intelligence),
                Addend::new("magic_theory", magic_theory),
                Addend::new("technique", te),
                Addend::new("form", fo),
                Addend::new("aura", aura),
                Addend::new("lab_mod", mods.lab_mod),
            ];
            let deficient = mods.deficient(&technique, &form);
            let base = sum(&addends);
            let total = if deficient { halve(base) } else { base };
            let within_focus = mods.has_focus.then(|| {
                let focused = base + te.min(fo);
                if deficient { halve(focused) } else { focused }
            });
            out.push(LabTotal {
                technique: technique.clone(),
                form: form.clone(),
                addends,
                total,
                within_focus,
                deficient,
            });
        }
    }
    out
}

// --- Casting totals (per Technique × Form) ---------------------------------

/// The within-focus counterparts of a [`CastingTotal`]'s four cast types (the
/// lower applicable Art added again before halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastingWithinFocus {
    /// The doubled lower Art score added within the focus.
    pub focus_art: i32,
    /// Within-focus formulaic total.
    pub formulaic: i32,
    /// Within-focus ritual total.
    pub ritual: i32,
    /// Within-focus fatiguing-spontaneous total.
    pub spontaneous_fatiguing: i32,
    /// Within-focus non-fatiguing-spontaneous total.
    pub spontaneous_non_fatiguing: i32,
}

/// The non-standard-casting variants of a cell's **Formulaic** Casting Total:
/// casting with no voice ("silent") and/or no gestures ("still"). The Words and
/// Gestures penalties apply to Formulaic and Spontaneous casting, never to Ritual
/// (Core:9236); these variants adjust the Formulaic total. Quiet Magic reduces the
/// no-voice penalty, Subtle Magic the no-gesture penalty, and Deft Form waives
/// both for spells in its Form; each residual penalty clamps at 0. Source:
/// Core:9236-9245, :4822-4826, :5073-5076, :3645-3648.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonStandardCasting {
    /// Residual no-voice penalty (≤ 0) after Quiet Magic / Deft Form.
    pub voice_penalty: i32,
    /// Residual no-gesture penalty (≤ 0) after Subtle Magic / Deft Form.
    pub gesture_penalty: i32,
    /// Formulaic total cast with no voice: `formulaic + voice_penalty`.
    pub silent: i32,
    /// Formulaic total cast with no gestures: `formulaic + gesture_penalty`.
    pub still: i32,
    /// Formulaic total cast with neither voice nor gestures.
    pub silent_and_still: i32,
    /// Whether Deft Form applies to this cell's Form (both penalties waived).
    pub deft_form: bool,
}

/// The Casting Total for one `(Technique, Form)` cell, split into the four cast
/// types from one shared breakdown.
///
/// Casting Score = Technique + Form + Stamina − Encumbrance + Aura + flat
/// CastingTotalMod (per scope). Formulaic = the score; Ritual = the score +
/// Artes Liberales + Philosophiae; fatiguing Spontaneous = ÷2; non-fatiguing
/// Spontaneous = ÷5. Within a Magical Focus the lower Art is added again; a
/// Deficient Art halves the totals. Source: Core:9089 (Casting Score), :9103-9145
/// (cast types), :4524-4527 (Method Caster), :4399-4422 (focus), :5909-5915
/// (Deficient halving).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastingTotal {
    /// Technique Art id of this cell.
    pub technique: Id,
    /// Form Art id of this cell.
    pub form: Id,
    /// The shared Casting-Score breakdown (technique, form, stamina, encumbrance, aura).
    pub addends: Vec<Addend>,
    /// The ritual-only extra addends (artes_liberales, philosophiae).
    pub ritual_addends: Vec<Addend>,
    /// Formulaic Casting Total.
    pub formulaic: i32,
    /// Ritual Casting Total (+ Artes Liberales + Philosophiae).
    pub ritual: i32,
    /// Fatiguing spontaneous total (÷2).
    pub spontaneous_fatiguing: i32,
    /// Non-fatiguing spontaneous total (÷5).
    pub spontaneous_non_fatiguing: i32,
    /// The within-focus variants; `None` when the magus holds no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<CastingWithinFocus>,
    /// The non-standard-casting (silent / still) variants of the Formulaic total.
    pub non_standard: NonStandardCasting,
    /// Whether a Deficient Art halved these totals.
    pub deficient: bool,
}

/// The formulaic casting score of one `(Technique, Form)` cell, without the die,
/// including the focus double when `focus` is set and the Deficient-Art halving.
/// Shared by the grid and by per-spell penetration.
fn formulaic_casting_score(
    entity: &Entity,
    ruleset: &Ruleset,
    mods: &InPlayMods,
    technique: &Id,
    form: &Id,
    focus: bool,
) -> i32 {
    let te = effective_art_score(entity, ruleset, technique);
    let fo = effective_art_score(entity, ruleset, form);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let enc = encumbrance(entity, ruleset).total;
    let mut score =
        te + fo + stamina - enc + entity.aura + mods.casting_mod_for(CastType::Formulaic);
    if focus {
        score += te.min(fo);
    }
    if mods.deficient(technique, form) {
        score = halve(score);
    }
    score
}

/// Casting Totals for every `(Technique, Form)` pair. Source: Core:9089-9145.
pub fn casting_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<CastingTotal> {
    let mods = in_play_mods(entity, ruleset);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let enc = encumbrance(entity, ruleset).total;
    let aura = entity.aura;
    let artes_liberales = ability(entity, ruleset, ID_ARTES_LIBERALES);
    let philosophiae = ability(entity, ruleset, ID_PHILOSOPHIAE);
    let weak_spont = mods.halvings.contains(&HalvableTotal::SpontaneousCasting);
    let mut out = Vec::new();
    for technique in ruleset.art_ids_of(ArtType::Technique) {
        let te = effective_art_score(entity, ruleset, &technique);
        for form in ruleset.art_ids_of(ArtType::Form) {
            let fo = effective_art_score(entity, ruleset, &form);
            let deficient = mods.deficient(&technique, &form);
            let addends = vec![
                Addend::new("technique", te),
                Addend::new("form", fo),
                Addend::new("stamina", stamina),
                Addend::new("encumbrance", -enc),
                Addend::new("aura", aura),
            ];
            let ritual_addends = vec![
                Addend::new("artes_liberales", artes_liberales),
                Addend::new("philosophiae", philosophiae),
            ];
            let common = sum(&addends);
            let focus_art = te.min(fo);

            let variant = |focused: bool| -> CastingScores {
                let focus_add = if focused { focus_art } else { 0 };
                let formulaic = post(
                    common + focus_add + mods.casting_mod_for(CastType::Formulaic),
                    deficient,
                    false,
                );
                let ritual = post(
                    common
                        + focus_add
                        + sum(&ritual_addends)
                        + mods.casting_mod_for(CastType::Ritual),
                    deficient,
                    false,
                );
                let spont_base = post(
                    common + focus_add + mods.casting_mod_for(CastType::Spontaneous),
                    deficient,
                    weak_spont,
                );
                CastingScores {
                    formulaic,
                    ritual,
                    spontaneous_fatiguing: halve(spont_base),
                    spontaneous_non_fatiguing: spont_base / 5,
                }
            };

            let base = variant(false);
            let voice_penalty = mods.residual_voice_penalty(&form);
            let gesture_penalty = mods.residual_gesture_penalty(&form);
            let non_standard = NonStandardCasting {
                voice_penalty,
                gesture_penalty,
                silent: base.formulaic + voice_penalty,
                still: base.formulaic + gesture_penalty,
                silent_and_still: base.formulaic + voice_penalty + gesture_penalty,
                deft_form: mods.deft_forms.contains(&form),
            };
            let within_focus = mods.has_focus.then(|| {
                let f = variant(true);
                CastingWithinFocus {
                    focus_art,
                    formulaic: f.formulaic,
                    ritual: f.ritual,
                    spontaneous_fatiguing: f.spontaneous_fatiguing,
                    spontaneous_non_fatiguing: f.spontaneous_non_fatiguing,
                }
            });

            out.push(CastingTotal {
                technique: technique.clone(),
                form: form.clone(),
                addends,
                ritual_addends,
                formulaic: base.formulaic,
                ritual: base.ritual,
                spontaneous_fatiguing: base.spontaneous_fatiguing,
                spontaneous_non_fatiguing: base.spontaneous_non_fatiguing,
                within_focus,
                non_standard,
                deficient,
            });
        }
    }
    out
}

/// Applies the Deficient-Art halving and (for spontaneous) the Weak-Spontaneous
/// halving to a casting score, in that order. Source: Core:5909-5915, :7060-7063.
fn post(score: i32, deficient: bool, weak_spont: bool) -> i32 {
    let mut s = score;
    if deficient {
        s = halve(s);
    }
    if weak_spont {
        s = halve(s);
    }
    s
}

struct CastingScores {
    formulaic: i32,
    ritual: i32,
    spontaneous_fatiguing: i32,
    spontaneous_non_fatiguing: i32,
}

// --- Penetration (per known spell) -----------------------------------------

/// A Penetration line for one known spell.
///
/// Penetration Total = Casting Total − Spell Level + Penetration Ability score
/// (Core:9159-9161). Weak Magic halves the Penetration Total *after* subtracting
/// the level (Core:7064-7067).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PenetrationLine {
    /// The known spell's id.
    pub spell: Id,
    /// The chosen parameter of a parameterized spell (the target `(Form)` of a
    /// meta-magic Vim spell), disambiguating two instances of one spell id that
    /// differ only by parameter. `None` for ordinary, unparameterized spells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The spell's (resolved) level.
    pub level: u32,
    /// The formulaic Casting Total used (base, no focus).
    pub casting_total: i32,
    /// The Penetration Ability score contributing to the bonus.
    pub penetration_ability: i32,
    /// The Penetration Total (base, no focus).
    pub total: i32,
    /// The within-focus Penetration Total; `None` when no Magical Focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus: Option<i32>,
    /// Whether Weak Magic halved the total.
    pub weak_magic: bool,
}

/// Per-known-spell Penetration Totals. Source: Core:9159-9161, :7064-7067.
pub fn penetration(entity: &Entity, ruleset: &Ruleset) -> Vec<PenetrationLine> {
    let mods = in_play_mods(entity, ruleset);
    let pen_ability = ability(entity, ruleset, ID_PENETRATION);
    let weak_magic = mods.halvings.contains(&HalvableTotal::Penetration);
    let mut out = Vec::new();
    for sel in &entity.spells {
        let Some(spell) = ruleset.spell(&sel.spell) else {
            continue;
        };
        let Some(level) = resolved_spell_level(sel, ruleset) else {
            continue;
        };
        let level_i = i32::try_from(level).unwrap_or(i32::MAX);
        let pen = |casting: i32| -> i32 {
            let raw = casting - level_i + pen_ability;
            if weak_magic { halve(raw) } else { raw }
        };
        let base_casting =
            formulaic_casting_score(entity, ruleset, &mods, &spell.technique, &spell.form, false);
        let within_focus = mods.has_focus.then(|| {
            let focus_casting = formulaic_casting_score(
                entity,
                ruleset,
                &mods,
                &spell.technique,
                &spell.form,
                true,
            );
            pen(focus_casting)
        });
        out.push(PenetrationLine {
            spell: sel.spell.clone(),
            parameter: sel.parameter.clone(),
            level,
            casting_total: base_casting,
            penetration_ability: pen_ability,
            total: pen(base_casting),
            within_focus,
            weak_magic,
        });
    }
    out
}

// --- Magic Resistance (per Form) -------------------------------------------

/// A per-Form Magic Resistance line.
///
/// A magus's Magic Resistance = Form + 5 × Parma Magica (Core:9390-9398). A
/// supernatural being uses its **Might Score** as a blanket resistance instead of
/// Parma — the two do not stack; the higher is the base (RoP:Magic:1472;
/// Core:2627), and the Form bonus is compatible with either. Limited Magic
/// Resistance drops the Form bonus; Flawed Parma / Weak Magic Resistance halve it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicResistance {
    /// The Form Art id.
    pub form: Id,
    /// The labelled breakdown (form, and one of parma/might).
    pub addends: Vec<Addend>,
    /// The Magic Resistance total.
    pub total: i32,
}

/// Per-Form Magic Resistance. Source: Core:9390-9398, :6142-6145, :6346-6349;
/// RoP:Magic:1472 (Might grants MR = Might Score, not stacking with Parma).
pub fn magic_resistance(entity: &Entity, ruleset: &Ruleset) -> Vec<MagicResistance> {
    let mods = in_play_mods(entity, ruleset);
    let parma = ability(entity, ruleset, ID_PARMA_MAGICA);
    let parma_mr = 5 * parma;
    // A Might-being's blanket resistance = its effective Might Score. Might and
    // Parma do not stack; the higher is the base (RoP:Magic:1472, Core:2627).
    let might = crate::effective::effective_might(entity, ruleset)
        .map(|m| i32::from(m.score))
        .unwrap_or(0);
    let no_form = mods.mr_mods.contains(&MagicResistanceEffect::NoFormBonus);
    let halved = mods.halvings.contains(&HalvableTotal::MagicResistance);
    let mut out = Vec::new();
    for form in ruleset.art_ids_of(ArtType::Form) {
        let fo = effective_art_score(entity, ruleset, &form);
        let form_bonus = if no_form { 0 } else { fo };
        let base_addend = if might > parma_mr {
            Addend::new("might", might)
        } else {
            Addend::new("parma", parma_mr)
        };
        let addends = vec![Addend::new("form", form_bonus), base_addend];
        let mut total = sum(&addends);
        if halved {
            total = halve(total);
        }
        out.push(MagicResistance {
            form,
            addends,
            total,
        });
    }
    out
}

// --- Encumbrance -----------------------------------------------------------

/// The Encumbrance read-out: total Load, Burden, and the Encumbrance penalty.
///
/// Burden comes from the Load table (Core:17103-17123); Encumbrance =
/// max(0, Burden − max(0, Strength)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncumbranceTotal {
    /// Total Load of all carried equipment.
    pub load: u32,
    /// Burden derived from the Load table.
    pub burden: i32,
    /// The Encumbrance penalty (≥ 0).
    pub total: i32,
}

/// The Load-table thresholds: index = Burden, value = Load at which that Burden
/// begins (Core:17103-17123).
const LOAD_TABLE: [u32; 11] = [0, 1, 3, 6, 10, 15, 21, 28, 36, 45, 55];

/// Burden for a total Load: the highest table index whose threshold is ≤ load.
fn burden_for_load(load: u32) -> i32 {
    let mut burden = 0;
    for (b, threshold) in LOAD_TABLE.iter().enumerate() {
        if load >= *threshold {
            burden = b as i32;
        }
    }
    burden
}

/// The character's Encumbrance. All carried equipment (equipped or not) counts
/// toward Load. Source: Core:17103-17123.
pub fn encumbrance(entity: &Entity, ruleset: &Ruleset) -> EncumbranceTotal {
    let load: u32 = entity
        .equipment
        .iter()
        .map(|slot| equipment_load(ruleset, &slot.item))
        .sum();
    let burden = burden_for_load(load);
    let strength = characteristic(entity, ruleset, Characteristic::Str);
    let total = (burden - strength.max(0)).max(0);
    EncumbranceTotal {
        load,
        burden,
        total,
    }
}

/// The Load of one equipment id (weapon, shield, or armor); 0 if unknown.
fn equipment_load(ruleset: &Ruleset, id: &Id) -> u32 {
    if let Some(w) = ruleset.weapon(id) {
        u32::from(w.load)
    } else if let Some(s) = ruleset.shield(id) {
        u32::from(s.load)
    } else if let Some(a) = ruleset.armor_item(id) {
        u32::from(a.load)
    } else {
        0
    }
}

// --- Combat ----------------------------------------------------------------

/// One combat line for an equipped weapon, with any equipped shield's modifiers
/// combined in (Core:16656) — **unless** the weapon is two-handed, which receives
/// no shield modifiers (Core:7494). Attack / Damage are `None` for a weapon that
/// lacks them (Dodge). Initiative is always reduced by Encumbrance (Core:16658);
/// Attack and Defense are reduced only when the Encumbrance is **not** largely due
/// to weapons and armor (Core:17105) — see [`combat_encumbrance_applies`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatLine {
    /// The weapon id.
    pub weapon: Id,
    /// The combat Ability id the weapon uses.
    pub ability: Id,
    /// Initiative total (Qik + weapon/shield Init − Encumbrance + CombatMod).
    pub initiative: i32,
    /// Attack total; `None` if the weapon has no attack (Dodge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<i32>,
    /// Defense total.
    pub defense: i32,
    /// Damage total; `None` if the weapon has no damage (Dodge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
    /// The weapon's Range in paces (missile / thrown); `None` for melee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<u16>,
}

/// Combat lines: one per equipped weapon, combining every equipped shield's
/// Init/Atk/Def modifiers. Source: Core:16658-16670, :16656, :17105.
pub fn combat_totals(entity: &Entity, ruleset: &Ruleset) -> Vec<CombatLine> {
    let mods = in_play_mods(entity, ruleset);
    let quickness = characteristic(entity, ruleset, Characteristic::Qik);
    let dexterity = characteristic(entity, ruleset, Characteristic::Dex);
    let strength = characteristic(entity, ruleset, Characteristic::Str);
    let enc = encumbrance(entity, ruleset).total;

    // Sum every equipped shield's modifiers into one combined shield line.
    let (shield_init, shield_attack, shield_defense) = entity
        .equipment
        .iter()
        .filter(|s| s.equipped)
        .filter_map(|s| ruleset.shield(&s.item))
        .fold((0i32, 0i32, 0i32), |(i, a, d), sh| {
            (
                i + i32::from(sh.init_mod),
                a + i32::from(sh.attack_mod),
                d + i32::from(sh.defense_mod),
            )
        });

    let cm = |stat: CombatStat| mods.combat_mods.get(&stat).copied().unwrap_or(0);

    // Attack/Defense take the Encumbrance penalty only when the load is NOT
    // largely weapons and armor; Initiative always takes it (Core:17105, :16658).
    let atk_def_enc = if combat_encumbrance_applies(entity, ruleset) {
        enc
    } else {
        0
    };

    let mut out = Vec::new();
    for slot in entity.equipment.iter().filter(|s| s.equipped) {
        let Some(weapon) = ruleset.weapon(&slot.item) else {
            continue;
        };
        // A two-handed weapon cannot be paired with a shield, so it takes none of
        // the combined shield modifiers (Core:7494).
        let (line_shield_init, line_shield_attack, line_shield_defense) = if weapon.two_handed {
            (0, 0, 0)
        } else {
            (shield_init, shield_attack, shield_defense)
        };
        // Ability specialization (+1) applies to Attack and Defense only, when the
        // slot is flagged and the weapon's Ability carries a specialty aligned to
        // this weapon (Core:7122, :7139). It acts as if the score were one higher.
        let spec_bonus = i32::from(specialization_bonus(entity, ruleset, slot, weapon));
        let combat_ability =
            effective_ability_score(entity, ruleset, &weapon.ability, None) + spec_bonus;
        let initiative = quickness + i32::from(weapon.init_mod) + line_shield_init - enc
            + cm(CombatStat::Initiative);
        let attack = weapon.attack_mod.map(|m| {
            dexterity + combat_ability + i32::from(m) + line_shield_attack - atk_def_enc
                + cm(CombatStat::Attack)
        });
        let defense =
            quickness + combat_ability + i32::from(weapon.defense_mod) + line_shield_defense
                - atk_def_enc
                + cm(CombatStat::Defense);
        let damage = weapon
            .damage_mod
            .map(|m| strength + i32::from(m) + cm(CombatStat::Damage));
        out.push(CombatLine {
            weapon: slot.item.clone(),
            ability: weapon.ability.clone(),
            initiative,
            attack,
            defense,
            damage,
            range: weapon.range,
        });
    }
    out
}

/// The Ability-specialization bonus for one equipped weapon slot: +1 when the
/// slot is flagged `specialization_applies` AND the entity holds a non-empty
/// specialty on the weapon's combat Ability (so the toggle is not a dead switch).
/// The specialty is a per-weapon alignment the player asserts — the engine never
/// matches specialty text to weapon names — so a flagged slot with a real specialty
/// grants the bonus. Source: Core Rules.md:7122 (Single Weapon longsword example),
/// :7139 ("Add +1 when using an Ability's specialization").
fn specialization_bonus(
    entity: &Entity,
    _ruleset: &Ruleset,
    slot: &crate::types::EquipmentSlot,
    weapon: &crate::equipment::Weapon,
) -> u8 {
    if !slot.specialization_applies {
        return 0;
    }
    let has_specialty = entity.ability_scores.iter().any(|a| {
        a.ability == weapon.ability && a.specialty.as_deref().is_some_and(|s| !s.trim().is_empty())
    });
    u8::from(has_specialty)
}

/// Total Load from combat gear — every carried weapon, shield, and armor, whether
/// equipped or not (a spare weapon is still a weapon). Classified by catalogue
/// item type via the same dispatch [`equipment_load`] uses. Source: Core:17105
/// ("weapons and armor"), :17107 (Load counts all carried gear).
fn combat_gear_load(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .equipment
        .iter()
        .filter(|slot| {
            ruleset.weapon(&slot.item).is_some()
                || ruleset.shield(&slot.item).is_some()
                || ruleset.armor_item(&slot.item).is_some()
        })
        .map(|slot| equipment_load(ruleset, &slot.item))
        .sum()
}

/// Whether combat gear makes up "largely" (the majority) of the total carried
/// Load, i.e. combat-gear Load ≥ half of total Load. `>= half` is our reading of
/// the rules' "largely due to weapons and armor" (Core:17105); documented in
/// RULES.md. Zero total Load is trivially a majority (nothing to penalize).
fn combat_gear_is_majority(combat_load: u32, total_load: u32) -> bool {
    // combat_load * 2 >= total_load, i.e. combat_load >= total_load / 2, without
    // integer-division rounding.
    combat_load.saturating_mul(2) >= total_load
}

/// Whether the Encumbrance penalty applies to Attack and Defense. The penalty is
/// waived ("Attack and Defense are not [penalized]") when the Encumbrance is
/// largely due to weapons and armor; otherwise it applies. Initiative is always
/// penalized regardless (Core:16658), so this governs only Attack/Defense.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:17105.
///
/// Crate-internal: an implementation detail of [`combat_totals`], not part of the
/// curated public API (unlike the surfaced totals `combat_totals` / `soak` /
/// `encumbrance` the frontend consumes).
pub(crate) fn combat_encumbrance_applies(entity: &Entity, ruleset: &Ruleset) -> bool {
    let total_load = encumbrance(entity, ruleset).load;
    let combat_load = combat_gear_load(entity, ruleset);
    !combat_gear_is_majority(combat_load, total_load)
}

// --- Soak ------------------------------------------------------------------

/// The Soak read-out.
///
/// Soak = Stamina + Armor Protection + SoakMod (Tough +3) + Bronze cord
/// (Core:16667, :10840-10844). The magus Form bonus is situational and shown as an
/// entered addend of 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoakTotal {
    /// The labelled breakdown (stamina, armor, soak_mod, bronze_cord, form_bonus).
    pub addends: Vec<Addend>,
    /// The Soak total.
    pub total: i32,
}

/// The character's Soak. Source: Core:16667, :5145-5147 (Tough), :10840-10844
/// (Bronze cord).
pub fn soak(entity: &Entity, ruleset: &Ruleset) -> SoakTotal {
    let mods = in_play_mods(entity, ruleset);
    let stamina = characteristic(entity, ruleset, Characteristic::Sta);
    let armor: i32 = entity
        .equipment
        .iter()
        .filter(|s| s.equipped)
        .filter_map(|s| ruleset.armor_item(&s.item))
        .map(|a| i32::from(a.protection))
        .sum();
    let bronze = entity
        .familiar
        .as_ref()
        .map(|f| i32::from(f.cord_bronze))
        .unwrap_or(0);
    let addends = vec![
        Addend::new("stamina", stamina),
        Addend::new("armor", armor),
        Addend::new("soak_mod", mods.soak_mod),
        Addend::new("bronze_cord", bronze),
        Addend::new("form_bonus", 0),
    ];
    let total = sum(&addends);
    SoakTotal { addends, total }
}

// --- Fatigue & Wounds ------------------------------------------------------

/// The five penalty-bearing Fatigue levels. A fixed rules taxonomy, rendered via
/// Fluent, never as a raw slug. Unconscious is a game state with no action penalty
/// and is intentionally not a variant here. Source: Core:17127-17129.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FatigueTier {
    /// No fatigue; no penalty.
    Fresh,
    /// One level lost; no penalty.
    Winded,
    /// Weary: −1 to all actions.
    Weary,
    /// Tired: −3 to all actions.
    Tired,
    /// Dazed: −5 to all actions.
    Dazed,
}

impl std::fmt::Display for FatigueTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FatigueTier::Fresh => "fresh",
            FatigueTier::Winded => "winded",
            FatigueTier::Weary => "weary",
            FatigueTier::Tired => "tired",
            FatigueTier::Dazed => "dazed",
        })
    }
}

/// A Fatigue level and the penalty it imposes on all actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FatigueLevel {
    /// The Fatigue tier this row reports; serializes to its stable slug
    /// (`"fresh"`, `"winded"`, `"weary"`, `"tired"`, `"dazed"`), mapped through
    /// Fluent. Unconscious is a game state and is intentionally not surfaced here.
    pub level: FatigueTier,
    /// The penalty applied at this level (≤ 0), after any HealthMod fatigue delta.
    pub penalty: i32,
}

/// The five penalty-bearing Fatigue levels and their penalties, adjusted by
/// HealthMod fatigue deltas (a positive delta reduces the penalty magnitude).
/// Unconscious is omitted (it is a state, not an action penalty). Source:
/// Core:17127-17129.
pub fn fatigue_levels(entity: &Entity, ruleset: &Ruleset) -> Vec<FatigueLevel> {
    let mods = in_play_mods(entity, ruleset);
    let delta = mods
        .health_mods
        .get(&HealthTrack::FatiguePenalty)
        .copied()
        .unwrap_or(0);
    // (id, base penalty). Fresh/Winded are penalty-free; Unconscious is its own
    // penalty (no numeric). Core:17127-17129 gives Weary −1, Tired −3, Dazed −5.
    [
        (FatigueTier::Fresh, 0),
        (FatigueTier::Winded, 0),
        (FatigueTier::Weary, -1),
        (FatigueTier::Tired, -3),
        (FatigueTier::Dazed, -5),
    ]
    .into_iter()
    .map(|(level, base)| FatigueLevel {
        level,
        // A positive delta reduces magnitude; never flip a penalty positive.
        penalty: (base + delta).min(0),
    })
    .collect()
}

/// The five wound bands, in ascending severity. A fixed rules taxonomy, rendered
/// via Fluent, never as a raw slug. Source: Core:17167-17191.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WoundBand {
    /// Light wound: −1 per wound.
    Light,
    /// Medium wound: −3 per wound.
    Medium,
    /// Heavy wound: −5 per wound.
    Heavy,
    /// Incapacitating wound (special; no numeric per-wound penalty).
    Incapacitating,
    /// Dead (special; open-ended top band).
    Dead,
}

impl std::fmt::Display for WoundBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            WoundBand::Light => "light",
            WoundBand::Medium => "medium",
            WoundBand::Heavy => "heavy",
            WoundBand::Incapacitating => "incapacitating",
            WoundBand::Dead => "dead",
        })
    }
}

/// One wound band's inclusive damage range and its per-wound penalty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WoundRange {
    /// The wound band; serializes to its stable slug (`"light"`, `"medium"`,
    /// `"heavy"`, `"incapacitating"`, `"dead"`), mapped through Fluent.
    pub level: WoundBand,
    /// Lowest damage-total value in this band.
    pub min: i32,
    /// Highest damage-total value in this band; `None` for the open-ended Dead band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    /// The per-wound penalty (≤ 0); `None` for Incapacitating / Dead (special).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub penalty: Option<i32>,
}

/// The Size-indexed wound ranges, with wound penalties adjusted by any HealthMod
/// wound delta. The band unit is `u = max(1, Size + 5)`; Light 1..u, Medium
/// u+1..2u, Heavy 2u+1..3u, Incapacitating 3u+1..4u, Dead 4u+1.. — so each +1 Size
/// widens every band (Core:17167-17191). Uses the character's derived Size.
pub fn wound_ranges(entity: &Entity, ruleset: &Ruleset) -> Vec<WoundRange> {
    let mods = in_play_mods(entity, ruleset);
    let delta = mods
        .health_mods
        .get(&HealthTrack::WoundPenalty)
        .copied()
        .unwrap_or(0);
    let size = crate::effective::size(entity, ruleset);
    let u = (size + 5).max(1);
    let pen = |base: i32| (base + delta).min(0);
    vec![
        WoundRange {
            level: WoundBand::Light,
            min: 1,
            max: Some(u),
            penalty: Some(pen(-1)),
        },
        WoundRange {
            level: WoundBand::Medium,
            min: u + 1,
            max: Some(2 * u),
            penalty: Some(pen(-3)),
        },
        WoundRange {
            level: WoundBand::Heavy,
            min: 2 * u + 1,
            max: Some(3 * u),
            penalty: Some(pen(-5)),
        },
        WoundRange {
            level: WoundBand::Incapacitating,
            min: 3 * u + 1,
            max: Some(4 * u),
            penalty: None,
        },
        WoundRange {
            level: WoundBand::Dead,
            min: 4 * u + 1,
            max: None,
            penalty: None,
        },
    ]
}

// --- Longevity -------------------------------------------------------------

/// What a Longevity Ritual made *today* would be worth — a suggestion, never the
/// stored value.
///
/// "+1 bonus for every five points or fraction of Creo Corpus Lab Total"
/// (Core:10662). Shown beside the entered-bonus input so a player who is creating
/// the ritual now (or reinventing it after an aging crisis, Core:10668, :10670) can
/// read off the number the rules give them. It moves whenever Creo, Corpus,
/// Intelligence, Magic Theory or the aura move — which is exactly why the *stored*
/// bonus must not be derived from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityHint {
    /// Today's Creo Corpus Lab Total, after any halvings.
    pub lab_total: i32,
    /// The bonus that Lab Total would buy: `ceil(lab_total / 5)`, floored at 0.
    pub suggested_bonus: i32,
    /// Whether a Deficient Art or Difficult Longevity Ritual halved `lab_total`.
    pub halved: bool,
}

/// The Longevity Ritual aging bonus read-out.
///
/// `bonus` is what the player entered, for **both** sources — the ritual is a past
/// event whose bonus was fixed by the Lab Total of the season it was made
/// (Core:10662, :10670), so nothing here is derived. `entered` distinguishes an
/// unfilled field from a deliberate 0. `hint` carries the live suggestion for a
/// self-made ritual only. The Bronze cord adds "to rolls to resist aging"
/// (Core:10844) and is noted separately, since it is not part of the ritual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityBonus {
    /// Whether the ritual is self-made or external.
    pub source: LongevitySource,
    /// The stored aging bonus (magnitude; applied as a negative to aging rolls).
    pub bonus: i32,
    /// Whether a bonus was actually entered; `false` ⇒ `bonus` is a placeholder 0.
    pub entered: bool,
    /// The Bronze-cord addition to aging-resistance (noted, not part of `bonus`).
    pub bronze_cord: i32,
    /// What a ritual made today would be worth; `None` for an external ritual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<LongevityHint>,
}

/// The Longevity Ritual read-out, or `None` when the magus has no ritual. Source:
/// Core:10662 (formula), :10668 + :10670 (the bonus is fixed at creation and only a
/// reinvention takes advantage of raised Arts), :10844 (Bronze cord).
pub fn longevity_bonus(entity: &Entity, ruleset: &Ruleset) -> Option<LongevityBonus> {
    let ritual = entity.longevity_ritual.as_ref()?;
    let bronze = entity
        .familiar
        .as_ref()
        .map(|f| i32::from(f.cord_bronze))
        .unwrap_or(0);
    // A hint only makes sense for a ritual this magus makes: an external one came
    // from another magus's Lab Total, which this sheet does not know (Core:10672).
    let hint = match ritual.source {
        LongevitySource::SelfMade => {
            let (lab_total, halved) = creo_corpus_lab_total(entity, ruleset);
            Some(LongevityHint {
                lab_total,
                suggested_bonus: suggested_longevity_bonus(lab_total),
                halved,
            })
        }
        LongevitySource::External => None,
    };
    Some(LongevityBonus {
        source: ritual.source,
        bonus: i32::from(ritual.bonus.unwrap_or(0)),
        entered: ritual.bonus.is_some(),
        bronze_cord: bronze,
        hint,
    })
}

/// The Creo Corpus Lab Total and whether it was halved.
///
/// "Your basic Lab Total is: Technique + Form + Intelligence + Magic Theory + Aura
/// Modifier" (Core:10276-10278) plus any flat Lab-Total modifier. The Aura Modifier
/// is a plain addend with no floor and no gate: a zero aura is simply "the absence
/// of aura, so powers used there function without hindrance" (Core:17658).
///
/// Two halvings can apply. A Deficient Creo or Corpus halves "almost all totals
/// (including … Lab Totals) to which a particular Form is added" (Core:5909-5915),
/// and Difficult Longevity Ritual makes anyone "creating a Longevity Ritual for you
/// … halve their Lab Total" (Core:5962-5964). **That the two compound is an
/// inference**: each Flaw halves the Lab Total and neither carves out the other, but
/// no passage states the interaction. The order is immaterial — [`halve`] truncates
/// toward zero — so it is fixed here as base → Deficient → Difficult.
fn creo_corpus_lab_total(entity: &Entity, ruleset: &Ruleset) -> (i32, bool) {
    let mods = in_play_mods(entity, ruleset);
    let base = characteristic(entity, ruleset, Characteristic::Int)
        + ability(entity, ruleset, ID_MAGIC_THEORY)
        + art(entity, ruleset, ID_CREO)
        + art(entity, ruleset, ID_CORPUS)
        + entity.aura
        + mods.lab_mod;
    let deficient = mods.deficient(&Id::new(ID_CREO), &Id::new(ID_CORPUS));
    let difficult = mods.halvings.contains(&HalvableTotal::LabLongevity);
    let mut total = base;
    if deficient {
        total = halve(total);
    }
    if difficult {
        total = halve(total);
    }
    (total, deficient || difficult)
}

/// The bonus a Creo Corpus Lab Total buys: "+1 bonus for every five points or
/// fraction" (Core:10662), i.e. `ceil(lab_total / 5)`. A non-positive Lab Total buys
/// nothing — there is no fraction of five points below one point.
fn suggested_longevity_bonus(lab_total: i32) -> i32 {
    if lab_total <= 0 {
        return 0;
    }
    // `i32::div_ceil` is still unstable, so round up in unsigned space.
    i32::try_from((lab_total as u32).div_ceil(5)).unwrap_or(i32::MAX)
}

// --- Masterpiece (lesser enchanted item cap) -------------------------------

/// The Masterpiece read-out: the best-Lab-Total lesser enchanted item cap.
///
/// The Masterpiece Virtue lets the magus keep one *lesser enchanted item* he
/// designed "based on his Lab Totals at character generation, following the
/// regular rules for construction of such a device" (Core:4476-4479). The
/// regular lesser-enchantment rule caps a single-season instillation at
/// `Lab Total ≥ 2 × effect level`, i.e. the effect level may not exceed
/// `Lab Total ÷ 2` (Core:10410). Vis costs are ignored (the parens provided
/// them), so the only bound the engine can honestly compute is that Lab-Total
/// cap. The best base `(Technique, Form)` Lab Total is used — the magus is free
/// to pick the Technique/Form that maximises it — without any Magical-Focus
/// doubling (a focus applies only to items within its narrow field).
///
/// This is **read-only guidance**: the engine does not auto-create a device or
/// spend an item-level budget. The player still enters the actual item under
/// Magic Items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterpieceCap {
    /// The Technique Art of the best Lab Total.
    pub technique: Id,
    /// The Form Art of the best Lab Total.
    pub form: Id,
    /// The best base `(Technique, Form)` Lab Total (no focus doubling).
    pub lab_total: i32,
    /// The maximum lesser-enchantment effect level: `lab_total ÷ 2` (Core:10410).
    pub cap: i32,
}

/// The Masterpiece lesser-item cap, or `None` when the magus lacks the Virtue.
/// Source: Core:4476-4479 (Virtue), :10410 (lesser-enchantment cap).
pub fn masterpiece_item_cap(entity: &Entity, ruleset: &Ruleset) -> Option<MasterpieceCap> {
    if !in_play_mods(entity, ruleset).has_masterpiece {
        return None;
    }
    // Best base Lab Total across the grid; the magus picks the Te/Fo that maxes it.
    lab_totals(entity, ruleset)
        .into_iter()
        .max_by_key(|lt| lt.total)
        .map(|lt| MasterpieceCap {
            technique: lt.technique,
            form: lt.form,
            lab_total: lt.total,
            cap: halve(lt.total),
        })
}

// --- Talisman (enchantment capacity) ---------------------------------------

/// The talisman's enchantment-capacity read-out, in pawns of Vim vis.
///
/// "The capacity of a talisman is independent of its shape and material, and
/// instead depends on the power of the magus to whom it is attuned. The maximum
/// number of pawns of Vim vis that may be used to prepare a talisman is equal to
/// the sum of the magus's highest Technique and highest Form" (Core:10619).
///
/// The two contributing Arts and their scores are surfaced alongside the sum so
/// the UI can show the whole derivation without doing arithmetic in JS.
///
/// This is **read-only guidance**, like [`MasterpieceCap`]: no `ValidationIssue`
/// is ever raised from it. Vis costs are out of scope — the model holds no vis
/// stock, so the engine cannot know how much of the capacity is actually opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalismanCapacity {
    /// The magus's highest Technique.
    pub technique: Id,
    /// The magus's highest Form.
    pub form: Id,
    /// That Technique's effective score.
    pub technique_score: i32,
    /// That Form's effective score.
    pub form_score: i32,
    /// The capacity: `technique_score + form_score`, in pawns of Vim vis.
    pub pawns: i32,
}

/// The Art of `art_type` with the highest **effective** score, and that score.
/// Ties go to the alphabetically first id, because [`Ruleset::art_ids_of`] is
/// sorted and the fold keeps the first strict maximum — so the read-out never
/// flickers between equal Arts. `None` only when the ruleset defines no Art of
/// that class.
fn highest_art(entity: &Entity, ruleset: &Ruleset, art_type: ArtType) -> Option<(Id, i32)> {
    ruleset
        .art_ids_of(art_type)
        .into_iter()
        .map(|id| {
            let score = crate::effective::effective_art_score(entity, ruleset, &id);
            (id, score)
        })
        .reduce(|best, current| if current.1 > best.1 { current } else { best })
}

/// The magus's talisman capacity, or `None` when he has no talisman.
///
/// Uses the per-Art maxima of **effective** scores (Puissant Art and the like
/// folded in by [`crate::effective::effective_art_score`]), *not* the best
/// [`lab_totals`] pair: the rule reads the magus's Art scores directly, and a
/// Deficient Art halves *totals*, never the score. A magus who has bought no Arts
/// still gets a read-out, at 0 pawns.
///
/// **Non-goal — instilled effects are charged against no budget.** A talisman's
/// [`crate::types::TalismanEffect`] levels are deliberately NOT added to
/// `item_level_used`, so they can never overrun `item_level_budget`. That budget
/// exists only because of two Redcap-only Virtues: Magic Items requires "You must
/// be a Redcap to take this Virtue" (Core:4347-4349, the requirement on `:4349`),
/// and the Redcap Social Status itself grants the fifty starting levels
/// (`:4842-4850`) while stating "You may not take The Gift" (`:4850`). A talisman
/// can only be attuned by a magus, so the budget can never fund one, and charging
/// against it would invent a limit the rules do not impose. The talisman's own
/// limit is this vis capacity, which the model cannot enforce (it holds no vis
/// stock) and therefore only reports.
///
/// Source: `Ars Magica - Definitive Edition (Core Rules).md:10619`.
pub fn talisman_capacity(entity: &Entity, ruleset: &Ruleset) -> Option<TalismanCapacity> {
    entity.talisman.as_ref()?;
    let (technique, technique_score) = highest_art(entity, ruleset, ArtType::Technique)?;
    let (form, form_score) = highest_art(entity, ruleset, ArtType::Form)?;
    Some(TalismanCapacity {
        technique,
        form,
        technique_score,
        form_score,
        pawns: technique_score.saturating_add(form_score),
    })
}

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
    /// The modifier amount (0 when the family is a mode toggle, e.g. Unaging).
    pub amount: i32,
}

/// Every surfaced-only modifier the character carries, for the read-out list.
/// Source: Core:3422-3425 (Apt Student), :5187-5190 (Unaging), :3645-3682
/// (non-standard casting).
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
    /// Whether the character is a magus (magic totals are present only then).
    pub is_magus: bool,
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
    /// One combat line per equipped weapon.
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
/// magus (per the type profile's `is_magus`).
pub fn derived_totals(entity: &Entity, ruleset: &Ruleset) -> DerivedTotals {
    let is_magus = ruleset
        .profile(&entity.type_id)
        .map(|p| p.is_magus)
        .unwrap_or(false);
    // A supernatural being (Might Score) has Magic Resistance too, even though it
    // is not a magus. Source: RoP:Magic:1472.
    let has_might = crate::effective::effective_might(entity, ruleset).is_some();
    DerivedTotals {
        is_magus,
        lab_totals: if is_magus {
            lab_totals(entity, ruleset)
        } else {
            Vec::new()
        },
        casting_totals: if is_magus {
            casting_totals(entity, ruleset)
        } else {
            Vec::new()
        },
        penetration: if is_magus {
            penetration(entity, ruleset)
        } else {
            Vec::new()
        },
        magic_resistance: if is_magus || has_might {
            magic_resistance(entity, ruleset)
        } else {
            Vec::new()
        },
        longevity: if is_magus {
            longevity_bonus(entity, ruleset)
        } else {
            None
        },
        masterpiece: if is_magus {
            masterpiece_item_cap(entity, ruleset)
        } else {
            None
        },
        talisman_capacity: if is_magus {
            talisman_capacity(entity, ruleset)
        } else {
            None
        },
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
        AbilityScore, ArtScore, EntityKind, EquipmentSlot, Familiar, LongevityRitual, RulesetRef,
        Selection, SpellSelection, Talisman,
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
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "casting_total_mod", "amount": 3, "scope": "formulaic_ritual" }] },
          { "id": "virtue.magical_focus", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "focus", "type": "ref", "domain": "text" }],
            "effects": [{ "type": "magical_focus", "param": "focus", "major": false }] },
          { "id": "flaw.deficient_technique", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "art", "type": "ref", "domain": "technique" }],
            "effects": [{ "type": "deficient_art", "param": "art" }] },
          { "id": "flaw.deficient_form", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "deficient_art", "param": "form" }] },
          { "id": "flaw.difficult_longevity_ritual", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "lab_longevity" }] },
          { "id": "virtue.tough", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "soak_mod", "amount": 3 }] },
          { "id": "flaw.weak_magic", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "penetration" }] },
          { "id": "flaw.flawed_parma", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "magic_resistance" }] },
          { "id": "virtue.enduring_constitution", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [
              { "type": "health_mod", "track": "wound_penalty", "amount": 1 },
              { "type": "health_mod", "track": "fatigue_penalty", "amount": 1 }
            ] },
          { "id": "virtue.apt_student", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "advancement_mod", "source": "taught", "amount": 5 }] },
          { "id": "virtue.quiet_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "quiet_words" }] },
          { "id": "virtue.subtle_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "subtle_gestures" }] },
          { "id": "virtue.deft_form", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "parameters": [{ "key": "form", "type": "ref", "domain": "form" }],
            "effects": [{ "type": "special_casting_mod", "kind": "deft_form", "param": "form" }] },
          { "id": "virtue.masterpiece", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "masterpiece_item" }] },
          { "id": "virtue.inventive_genius", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "lab_total_mod", "amount": 3 }] },
          { "id": "flaw.lame", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "combat_mod", "amount": -3, "target": "initiative" }] },
          { "id": "flaw.limited_magic_resistance", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_resistance_mod", "kind": "no_form_bonus" }] },
          { "id": "flaw.susceptibility_to_divine_power", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_resistance_mod", "kind": "susceptible_divine" }] },
          { "id": "virtue.unaging", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "aging_mod", "kind": "no_aging", "amount": 0 }] },
          { "id": "virtue.diedne_magic", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "diedne" }] },
          { "id": "virtue.life_boost", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "special_casting_mod", "kind": "life_boost" }] },
          { "id": "virtue.academic_concentration", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "parameters": [{ "key": "subject", "type": "ref", "domain": "text" }],
            "effects": [{ "type": "ability_roll_mod", "param": "subject", "amount": 3 }] },
          { "id": "flaw.weak_spontaneous", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "spontaneous_casting" }] },
          { "id": "virtue.long_winded", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
            "effects": [{ "type": "health_mod", "track": "fatigue_roll", "amount": 3 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "magus", "is_magus": true,
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "hermetic"], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "gift_policy": "required",
            "gift_categories": [], "creation_phases": ["concept"] },
          { "id": "grog", "is_magus": false,
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
          { "id": "ability.brawl", "category": "general" }
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
              "two_handed": true, "ability": "ability.single_weapon" }
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
    /// (Core:10670), so the engine must not overwrite it with today's derivation.
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
    /// its bonus came from another magus's Lab Total (Core:10672), which this
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
    /// Total" (Core:10662) — 35 → ceil(35/5) = 7, matching the book's worked
    /// example (Core:2488, :2573).
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
    /// Modifier as a plain addend (Core:10276-10278), and no aura simply means no
    /// hindrance (Core:17658) — 30 → ceil(30/5) = 6.
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
    /// 27 → ceil(27/5) = 6 (Core:10276-10278).
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

    /// Deficient Creo halves the Lab Total the hint reads (Core:5909-5915):
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
    /// Ritual for you must halve their Lab Total" (Core:5962-5964) — 35 → 17 → 4.
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
        assert_eq!(hint.lab_total, 8, "trunc(trunc(35/2)/2)");
        assert_eq!(hint.suggested_bonus, 2);
        assert!(hint.halved);
    }

    /// A non-positive Lab Total suggests no bonus at all — "every five points" has
    /// no meaning below one point (Core:10662).
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
    /// enchanted item the magus could make — level ≤ Lab Total ÷ 2 (Core:10410).
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
        let cap = masterpiece_item_cap(&e, &rs).expect("has masterpiece");
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
        assert!(masterpiece_item_cap(&e, &rs).is_none());
        assert!(derived_totals(&e, &rs).masterpiece.is_none());
    }

    /// A magus with a talisman: its capacity in pawns of Vim vis is his highest
    /// Technique + his highest Form (Core:10619). Creo 10 / Perdo 4 and Corpus 12 /
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
    /// pair: it "depends on the power of the magus" (Core:10619), and a Deficient
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
    /// plus Method Caster's +3 formulaic (Core:9089, :4524-4527, :4399-4422).
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
            equipped: false,
            specialization_applies: false,
        }];
        // Give the weapon Load 6 via a heavier item: use armor Load path instead.
        e.equipment = vec![EquipmentSlot {
            item: Id::new("armor.leather_scale"),
            equipped: false,
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

    /// A Deficient Technique halves every casting total using it (Core:5913-5915).
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
    /// gestures −5 off the Formulaic total (Core:9243-9245); combined −15.
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
        let f = cell.formulaic;
        let nc = &cell.non_standard;
        assert_eq!(nc.voice_penalty, -10);
        assert_eq!(nc.gesture_penalty, -5);
        assert_eq!(nc.silent, f - 10);
        assert_eq!(nc.still, f - 5);
        assert_eq!(nc.silent_and_still, f - 15);
        assert!(!nc.deft_form);
    }

    /// Quiet Magic cuts the no-voice penalty to −5; a second casting eliminates it
    /// (the residual clamps at 0). Core:4822-4826.
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
    /// Core:5073-5076.
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
    /// Core:3645-3648.
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

    /// Per-Form Magic Resistance = Form + 5 × Parma (Core:9390-9398).
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

    /// Flawed Parma halves Magic Resistance (Core:6142-6145).
    #[test]
    fn flawed_parma_halves_magic_resistance() {
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
        e.selections = vec![Selection::new(Id::new("flaw.flawed_parma"))];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // (4 + 15) / 2 = 9.
        assert_eq!(ignem.total, 9);
    }

    /// A supernatural being's Magic Resistance equals its Might Score, blanket
    /// across every Form; it does not stack with Parma — the higher is used
    /// (RoP:Magic:1472; Core:2627).
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

    /// Might and Parma do not stack: the base uses whichever is higher (Core:2627).
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

    /// Per-known-spell penetration = Casting Total − Level + Penetration score;
    /// Weak Magic halves after subtracting level (Core:9159-9161, :7064-7067).
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
        // Casting Total = Cr10 + Ig5 + Sta2 = 17; − level 20 + Penetration 4 = 1.
        assert_eq!(pen[0].casting_total, 17);
        assert_eq!(pen[0].total, 1);

        // With Weak Magic: (17 − 20 + 4)/2 = 0 (halve toward zero).
        e.selections = vec![Selection::new(Id::new("flaw.weak_magic"))];
        let pen = penetration(&e, &rs);
        assert!(pen[0].weak_magic);
        assert_eq!(pen[0].total, 0);
    }

    /// A parameterized meta-magic Vim spell's Casting Total and Penetration use
    /// its catalogue Vim Arts (MuVi), NOT the chosen target-Form parameter, and
    /// the Penetration line carries the chosen parameter so two instances of the
    /// one spell id are distinct. Source: Core Rules.md:15791-15794 (the (Form) is
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

    /// A grog combat line with a weapon and a shield combined (Core:16658-16670,
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
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("shield.round"),
                equipped: true,
                specialization_applies: false,
            },
        ];
        let lines = combat_totals(&e, &rs);
        assert_eq!(lines.len(), 1);
        let l = &lines[0];
        // Total Load 2 (sword 1 + shield 1) → Burden 1; Str 3 → Encumbrance 0.
        // Init = Qik 1 + WpnInit 2 + ShieldInit 0 − Enc 0 = 3.
        assert_eq!(l.initiative, 3);
        // Attack = Dex 2 + Ability 4 + WpnAtk 4 + ShieldAtk 0 = 10.
        assert_eq!(l.attack, Some(10));
        // Defense = Qik 1 + Ability 4 + WpnDef 1 + ShieldDef 2 = 8.
        assert_eq!(l.defense, 8);
        // Damage = Str 3 + WpnDam 6 = 9.
        assert_eq!(l.damage, Some(9));
    }

    /// Issue B: a two-handed weapon wielded with a shield gets NO shield
    /// Init/Attack/Defense modifiers, but the shield still adds to Load /
    /// Encumbrance (Core:7494, :17107).
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
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("shield.round"),
                equipped: true,
                specialization_applies: false,
            },
        ];
        // Total Load 2 (great sword) + 1 (shield) = 3 → Burden 2; Str 0 → Enc 2.
        assert_eq!(encumbrance(&e, &rs).total, 2);
        let lines = combat_totals(&e, &rs);
        assert_eq!(lines.len(), 1);
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
    /// Initiative (which do not use the Ability) are unchanged (Core:7122, :7139).
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
            equipped: true,
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

    /// Issue A: the pure "largely due to weapons and armor" majority test —
    /// combat-gear Load ≥ half of total Load exempts Attack/Defense (Core:17105).
    /// Documents the ">= half" interpretation of "largely" (RULES.md).
    #[test]
    fn combat_gear_majority_boundary() {
        // (i) majority combat gear (7 of 10) → exempt.
        assert!(combat_gear_is_majority(7, 10));
        // (ii) majority non-combat load (3 of 10) → NOT exempt (penalized).
        assert!(!combat_gear_is_majority(3, 10));
        // (iii) exact 50/50 → exempt (the ">= half" choice).
        assert!(combat_gear_is_majority(5, 10));
        // No load at all → trivially exempt (nothing to penalize).
        assert!(combat_gear_is_majority(0, 0));
    }

    /// Issue A: with all Load coming from combat gear (weapons + armor), the
    /// Encumbrance penalty is exempt from Attack/Defense but still hits Initiative
    /// (Core:17105, :16658).
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
                equipped: true,
                specialization_applies: false,
            },
            EquipmentSlot {
                item: Id::new("armor.leather_scale"),
                equipped: true,
                specialization_applies: false,
            },
        ];
        assert!(!combat_encumbrance_applies(&e, &rs));
        let l = &combat_totals(&e, &rs)[0];
        // Init = Qik 1 + WpnInit 2 − Enc 1 = 2 (Initiative IS penalized).
        assert_eq!(l.initiative, 2);
        // Attack = Dex 2 + Ability 4 + WpnAtk 4 = 10 (NO −Enc: exempt).
        assert_eq!(l.attack, Some(10));
        // Defense = Qik 1 + Ability 4 + WpnDef 1 = 6 (NO −Enc: exempt).
        assert_eq!(l.defense, 6);
    }

    /// Soak with Tough (+3) and a Bronze cord, plus worn armor (Core:16667,
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
            equipped: true,
            specialization_applies: false,
        }];
        let s = soak(&e, &rs);
        // Sta 2 + Armor 3 + Tough 3 + Bronze 2 + Form 0 = 10.
        assert_eq!(s.total, 10);
    }

    /// Encumbrance from a hand-set Load: armor Load 1 → Burden 1; Str 0 → Enc 1
    /// (Core:17103-17123).
    #[test]
    fn encumbrance_from_load_table() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Str, 0);
        e.equipment = vec![EquipmentSlot {
            item: Id::new("armor.leather_scale"),
            equipped: true,
            specialization_applies: false,
        }];
        let enc = encumbrance(&e, &rs);
        assert_eq!(enc.load, 1);
        assert_eq!(enc.burden, 1);
        assert_eq!(enc.total, 1);
    }

    /// Wound ranges for Size 0 and Size +1 (Core:17167-17180).
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

    /// Size +1 widens every band by the unit growth (Core:17167-17180).
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
    /// (Core:3751-3754).
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
    /// from `effective.rs`, not reimplemented (Core:16617, :16464-16475).
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
        assert!(!g.is_magus);
        assert!(g.lab_totals.is_empty());
        assert!(g.casting_totals.is_empty());
        let m = derived_totals(&magus(), &rs);
        assert!(m.is_magus);
        assert!(!m.lab_totals.is_empty());
        assert!(!m.casting_totals.is_empty());
    }

    /// Surfaced-only modifiers (Apt Student) are listed, not folded into a number.
    #[test]
    fn surfaced_modifiers_are_listed() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.apt_student"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::Advancement
            && m.detail == "taught"
            && m.amount == 5));
    }

    /// Inventive Genius folds a flat +3 into the Lab-Total `lab_mod` addend of
    /// every cell (Core:4151-4154).
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

    /// A CombatMod (Lame, −3 Initiative) folds into the Initiative combat total.
    #[test]
    fn combat_mod_adjusts_initiative() {
        let rs = ruleset();
        let mut e = grog();
        set_char(&mut e, Characteristic::Qik, 1);
        set_char(&mut e, Characteristic::Str, 3); // Str 3 → Encumbrance 0 with Load 1.
        e.equipment = vec![EquipmentSlot {
            item: Id::new("weapon.long_sword"),
            equipped: true,
            specialization_applies: false,
        }];
        // Baseline Init = Qik 1 + WpnInit 2 − Enc 0 = 3.
        assert_eq!(combat_totals(&e, &rs)[0].initiative, 3);
        e.selections = vec![Selection::new(Id::new("flaw.lame"))];
        // With Lame (−3 Initiative): 3 − 3 = 0.
        assert_eq!(combat_totals(&e, &rs)[0].initiative, 0);
    }

    /// Limited Magic Resistance (a MagicResistanceMod NoFormBonus) drops the Form
    /// contribution, leaving resistance from Parma alone (Core:6346-6349).
    #[test]
    fn limited_magic_resistance_drops_form_bonus() {
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
        e.selections = vec![Selection::new(Id::new("flaw.limited_magic_resistance"))];
        let mr = magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        // Form bonus dropped: 0 + 5 × Parma 3 = 15 (not 19).
        assert_eq!(ignem.total, 15);
        let form_addend = ignem.addends.iter().find(|a| a.label == "form").unwrap();
        assert_eq!(form_addend.value, 0);
    }

    /// An AgingMod (Unaging → no_aging) is surfaced with amount 0, not simulated.
    #[test]
    fn aging_mod_is_surfaced() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.unaging"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(
            s.iter().any(|m| m.family == ModifierFamily::Aging
                && m.detail == "no_aging"
                && m.amount == 0)
        );
    }

    /// The multi-variant SpecialCasting arm (Diedne + Life Boost) is surfaced
    /// labelled, one entry per variant, amount 0.
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
            && m.amount == 0));
        assert!(s.iter().any(|m| m.family == ModifierFamily::SpecialCasting
            && m.detail == "life_boost"
            && m.amount == 0));
    }

    /// A non-flat Magic-Resistance modifier (Susceptibility to Divine power) is
    /// surfaced labelled with amount 0, not silently dropped. Only NoFormBonus is
    /// folded into the flat per-Form MR number; the realm-conditional variants are
    /// listed. Source: Core:6815-6826.
    #[test]
    fn susceptibility_magic_resistance_is_surfaced() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new(
            "flaw.susceptibility_to_divine_power",
        ))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::MagicResistance
            && m.detail == "susceptible_divine"
            && m.amount == 0));
    }

    /// An AbilityRollMod (Academic Concentration) is surfaced with the free-text
    /// subject as its detail and the bonus as its amount (Core:3362-3367).
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
            && m.amount == 3));
    }

    /// Weak Spontaneous Magic halves the spontaneous totals only; the formulaic
    /// total is untouched (Core:7060-7063).
    #[test]
    fn weak_spontaneous_magic_halves_spontaneous_totals() {
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
        // spont base = halve(Cr10 + Ig5 + Sta2) = halve(17) = 8; fatiguing = 4,
        // non-fatiguing = 8/5 = 1.
        assert_eq!(cell.spontaneous_fatiguing, 4);
        assert_eq!(cell.spontaneous_non_fatiguing, 1);
        // Formulaic is not a spontaneous total and is not halved.
        assert_eq!(cell.formulaic, 17);
    }

    /// Deficient Art and Weak Spontaneous stack on spontaneous totals: halved
    /// twice; the formulaic total is halved once (Deficient only).
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
        // Spontaneous base = halve(halve(17)) = halve(8) = 4; fatiguing = 2,
        // non-fatiguing = 4/5 = 0.
        assert_eq!(cell.spontaneous_fatiguing, 2);
        assert_eq!(cell.spontaneous_non_fatiguing, 0);
    }

    /// The per-spell penetration path halves the casting score for a Deficient Art
    /// (`formulaic_casting_score`, Core:5913-5915).
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

    /// A surfaced-only health-roll track (Long-Winded → fatigue_roll +3) is listed
    /// under family "health_roll", and does not perturb the folded Fatigue
    /// penalties (Core:17127-17129).
    #[test]
    fn health_roll_track_is_surfaced_not_folded() {
        let rs = ruleset();
        let mut e = magus();
        e.selections = vec![Selection::new(Id::new("virtue.long_winded"))];
        let s = surfaced_modifiers(&e, &rs);
        assert!(s.iter().any(|m| m.family == ModifierFamily::HealthRoll
            && m.detail == "fatigue_roll"
            && m.amount == 3));
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
