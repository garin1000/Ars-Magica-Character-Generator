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
    CastingScope, CombatStat, Effect, Entity, Familiar, HalvableTotal, HealthTrack, Id,
    LongevitySource, MAX_CORD_SCORE, MagicResistanceEffect, SpecialCasting,
};

// --- Non-standard-casting penalty constants (Ars Magica - Definitive Edition
// (Core Rules).md:9243-9245) ---------------------------------------------

/// Casting-Score penalty for casting with **no voice** at all (the "None" Words
/// row). Source: Ars Magica - Definitive Edition (Core Rules).md:9245.
const NO_VOICE_PENALTY: i32 = -10;
/// Casting-Score penalty for casting with **no gestures** at all (the "None"
/// Gestures row). Source: Ars Magica - Definitive Edition (Core Rules).md:9245.
const NO_GESTURE_PENALTY: i32 = -5;
/// The no-voice-penalty reduction one casting of Quiet Magic grants (soft voice →
/// no penalty, no voice → −5, i.e. +5; a second casting eliminates it). Source:
/// Ars Magica - Definitive Edition (Core Rules).md:4822-4826.
const QUIET_MAGIC_VOICE_REDUCTION: i32 = 5;
/// The no-gesture-penalty reduction Subtle Magic grants (no gestures → no
/// penalty, i.e. +5). Source: Ars Magica - Definitive Edition (Core Rules).md:5073-5076.
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
/// Every casting/lab/penetration total folds in [`Entity::aura`], which is only
/// clamped to `AURA_MODIFIER_MIN..=AURA_MODIFIER_MAX` by `Entity::normalize`
/// (`types.rs`) — a value that reaches one of these totals *before* that pass
/// runs (a freshly deserialized save under `ValidationMode::Silent`, which still
/// computes derived totals per this engine's "one evaluation path") could
/// otherwise overflow a bare `i32` sum: silently wrapping to a nonsensical total
/// in a release build (`overflow-checks = false` is Cargo's release default),
/// or panicking in a debug build. `i64` cannot overflow summing any realistic
/// number of `i32` terms, so this is exact for every legal input and merely
/// clamps the display value for an illegal one — mirroring the
/// `saturating_add`/`i64`-widening pattern already used by
/// `effective/warping.rs::warping_points_total` and
/// `effective/xp.rs::charged_cost`.
fn saturating_i32_sum(terms: impl IntoIterator<Item = i32>) -> i32 {
    let total: i64 = terms.into_iter().map(i64::from).sum();
    i32::try_from(total).unwrap_or(if total > 0 { i32::MAX } else { i32::MIN })
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
                // Surfaced-only families: listed labelled, never simulated. Aging is
                // the exception — `aging.rs` consumes it (M6/6b6); it is listed here
                // as a standing modifier, not because nothing reads it.
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
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::LocalityAbilityCapFraction { .. }
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
    /// clamped so a Virtue can never turn it into a bonus. Source: Ars Magica -
    /// Definitive Edition (Core Rules).md:9245, :4822-4826, :3645-3648.
    fn residual_voice_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_VOICE_PENALTY + self.voice_reduction).min(0)
    }

    /// The residual Casting-Score penalty for casting a `form` spell with no
    /// gestures: Deft Form waives it, else the −5 base plus Subtle Magic reduction,
    /// clamped at 0. Source: Ars Magica - Definitive Edition (Core Rules).md:9245,
    /// :5073-5076, :3645-3648.
    fn residual_gesture_penalty(&self, form: &Id) -> i32 {
        if self.deft_forms.contains(form) {
            return 0;
        }
        (NO_GESTURE_PENALTY + self.gesture_reduction).min(0)
    }
}

/// The four ways a casting total is derived (Ars Magica - Definitive Edition (Core Rules).md:9095-9145). One breakdown yields
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
mod lab;

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
/// requires 75 points" (Ars Magica - Definitive Edition (Core Rules).md:10836).
///
/// A fixed five-entry rule curve, so it is a `const` here rather than ruleset data
/// — the same call as [`LOAD_TABLE`] for Encumbrance. RULES.md is its provenance
/// home.
const CORD_COST_TABLE: [u32; 6] = [0, 5, 15, 30, 50, 75];

/// The rules-legal score of a stored cord value, clamped to the +5 maximum
/// (Ars Magica - Definitive Edition (Core Rules).md:10836).
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
/// (Source: Ars Magica - Definitive Edition (Core Rules).md:10844)
///
/// The one entity-level accessor for that score, shared by every total the cord
/// feeds: [`soak`], the aging-resistance note on [`longevity_bonus`], and the
/// crisis-survival read-out. It goes **through** [`cord_score`], so the +5 maximum
/// (`:10836`) keeps its single home there and cannot be bypassed by adding a
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3422-3425 (Apt
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
    /// The familiar bonding read-out (magi with a familiar only; `None` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiar: Option<FamiliarReadout>,
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
/// magus (per the type profile's `is_magus`).
pub fn derived_totals(entity: &Entity, ruleset: &Ruleset) -> DerivedTotals {
    let is_magus = ruleset
        .profile(&entity.type_id)
        .map(|p| p.is_magus)
        .unwrap_or(false);
    // A supernatural being (Might Score) has Magic Resistance too, even though it
    // is not a magus. Source: Ars Magica 5e - Realms of Power - Magic.md:1472.
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
        familiar: if is_magus {
            familiar_readout(entity, ruleset)
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
    // Module-split (Wave 6) accessors: these two helpers moved to
    // `derived::combat` as domain-private code motion, but the existing
    // white-box tests below call them by bare name — `use super::*;` above
    // only re-imports `derived`'s own namespace, not a sibling submodule's,
    // so each needs an explicit import path (the one test-file change the
    // split's contract allows).
    use super::combat::{combat_encumbrance_applies, combat_gear_is_majority};
    use crate::ruleset::RulesetSources;
    use crate::types::{
        AbilityScore, ArtScore, EntityKind, EquipmentSlot, Familiar, LongevityRitual, MightScore,
        Realm, RulesetRef, Selection, SpellSelection, SupernaturalPower, Talisman,
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
            "effects": [{ "type": "magic_total_halving", "total": "magic_resistance" }] },
          { "id": "virtue.enduring_constitution", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [
              { "type": "health_mod", "track": "wound_penalty", "amount": 1 },
              { "type": "health_mod", "track": "fatigue_penalty", "amount": 1 }
            ] },
          { "id": "virtue.apt_student", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "advancement_mod", "source": "taught", "amount": 5 }] },
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
          { "id": "flaw.limited_magic_resistance", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_resistance_mod", "kind": "no_form_bonus" }] },
          { "id": "flaw.susceptibility_to_divine_power", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_resistance_mod", "kind": "susceptible_divine" }] },
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
            "effects": [{ "type": "ability_roll_mod", "param": "subject", "amount": 3 }] },
          { "id": "flaw.weak_spontaneous", "kind": "flaw", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
            "effects": [{ "type": "magic_total_halving", "total": "spontaneous_casting" }] },
          { "id": "virtue.long_winded", "kind": "virtue", "classification": "in_play_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "health_mod", "track": "fatigue_roll", "amount": 3 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] }
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
            life_stages: None,
            childhoods: None,
            aging: None,
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:10670), so the engine must not overwrite it with today's derivation.
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
    /// its bonus came from another magus's Lab Total (Ars Magica - Definitive Edition (Core Rules).md:10672), which this
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
    /// Total" (Ars Magica - Definitive Edition (Core Rules).md:10662) — 35 → ceil(35/5) = 7, matching the book's worked
    /// example (Ars Magica - Definitive Edition (Core Rules).md:2488, :2573).
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
    /// Modifier as a plain addend (Ars Magica - Definitive Edition (Core Rules).md:10276-10278), and no aura simply means no
    /// hindrance (Ars Magica - Definitive Edition (Core Rules).md:17658) — 30 → ceil(30/5) = 6.
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
    /// 27 → ceil(27/5) = 6 (Ars Magica - Definitive Edition (Core Rules).md:10276-10278).
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
    /// by `Entity::normalize` (`types.rs:2862`); a value that reaches
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

    /// Deficient Creo halves the Lab Total the hint reads (Ars Magica - Definitive Edition (Core Rules).md:5909-5915):
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
    /// Ritual for you must halve their Lab Total" (Ars Magica - Definitive Edition (Core Rules).md:5962-5964) — 35 → 17 → 4.
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
    /// no meaning below one point (Ars Magica - Definitive Edition (Core Rules).md:10662).
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
    /// enchanted item the magus could make — level ≤ Lab Total ÷ 2 (Ars Magica - Definitive Edition (Core Rules).md:10410).
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

    /// A Weak Enchanter designing his Masterpiece item is still "creating" an
    /// enchanted item (Ars Magica - Definitive Edition (Core Rules).md:4476-4479:
    /// the item is designed "following the regular rules for construction of such
    /// a device"), so the cap must derive from the halved `enchanting` figure
    /// (:7060-7063), not the un-halved `total` — otherwise the cap is double what
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
        let cap = masterpiece_item_cap(&e, &rs).expect("has masterpiece");
        assert_eq!(cap.lab_total, 17);
        assert_eq!(cap.cap, 8);
        assert_eq!(cap.technique.as_str(), "art.creo");
        assert_eq!(cap.form.as_str(), "art.corpus");
    }

    /// A magus with a talisman: its capacity in pawns of Vim vis is his highest
    /// Technique + his highest Form (Ars Magica - Definitive Edition (Core Rules).md:10619). Creo 10 / Perdo 4 and Corpus 12 /
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
    /// pair: it "depends on the power of the magus" (Ars Magica - Definitive Edition (Core Rules).md:10619), and a Deficient
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
        })
        .unwrap()
    }

    /// The magus profile of [`ruleset`], on its own.
    const MAGUS_PROFILE: &str = r#"[
      { "id": "magus", "is_magus": true,
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
          { "id": "grog", "is_magus": false,
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
            }],
        }
    }

    /// Cord scores are bought off a fixed 5-entry curve — +1 costs 5, +2 15, +3 30,
    /// +4 50, +5 75 — and the read-out is the **total** across all three cords
    /// (Ars Magica - Definitive Edition (Core Rules).md:10836).
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

    /// A cord score above the curve's top (+5 is the maximum, Ars Magica - Definitive Edition (Core Rules).md:10836) is
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

    /// The +5 cord maximum (Ars Magica - Definitive Edition (Core Rules).md:10836) is enforced in **one** place, so every
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
    /// maximum (Ars Magica - Definitive Edition (Core Rules).md:10836), because they share one accessor
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
    /// Size -2 and Magic Might 10 bind as a level 25 enchantment (Ars Magica - Definitive Edition (Core Rules).md:10824,
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

    /// Bond-invested power levels are summed for information only: "there is no
    /// limit to the number of powers which may be invested in a familiar"
    /// (Ars Magica - Definitive Edition (Core Rules).md:10866), so there is no budget to compare against and no issue to
    /// raise.
    #[test]
    fn familiar_invested_power_levels_sums_with_no_budget() {
        let mut f = statblock_familiar();
        assert_eq!(familiar_invested_power_levels(&f), 15);
        f.powers.push(SupernaturalPower {
            name: "Shapechanging".into(),
            level: 25,
        });
        assert_eq!(familiar_invested_power_levels(&f), 40);
    }

    /// The bonding Lab Total is the ordinary Lab Total shape (Ars Magica - Definitive Edition (Core Rules).md:10826), so the
    /// best `(Technique, Form)` cell of the existing grid is taken — and unlike
    /// Masterpiece, a **focus** may apply here (`:10818`), so the best cell's
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

        let out = familiar_readout(&e, &rs).expect("a familiar has a read-out");
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
        let out = familiar_readout(&e, &rs).expect("a familiar has a read-out");
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
        assert!(familiar_readout(&e, &rs).is_none());
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
        assert!(familiar_readout(&e, &rs).is_none());
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
    /// plus Method Caster's +3 formulaic (Ars Magica - Definitive Edition (Core Rules).md:9089, :4524-4527, :4399-4422).
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

    /// A Deficient Technique halves every casting total using it (Ars Magica - Definitive Edition (Core Rules).md:5913-5915).
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
    /// gestures −5 off the Formulaic total (Ars Magica - Definitive Edition (Core Rules).md:9243-9245); combined −15.
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
    /// (the residual clamps at 0). Ars Magica - Definitive Edition (Core Rules).md:4822-4826.
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
    /// Ars Magica - Definitive Edition (Core Rules).md:5073-5076.
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
    /// Ars Magica - Definitive Edition (Core Rules).md:3645-3648.
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

    /// Per-Form Magic Resistance = Form + 5 × Parma (Ars Magica - Definitive Edition (Core Rules).md:9390-9398).
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

    /// Flawed Parma halves Magic Resistance (Ars Magica - Definitive Edition (Core Rules).md:6142-6145).
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
    /// (Ars Magica 5e - Realms of Power - Magic.md:1472; Ars Magica - Definitive Edition (Core Rules).md:2627).
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

    /// Might and Parma do not stack: the base uses whichever is higher (Ars Magica - Definitive Edition (Core Rules).md:2627).
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
    /// Weak Magic halves after subtracting level (Ars Magica - Definitive Edition (Core Rules).md:9159-9161, :7064-7067).
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
    /// one spell id are distinct. Source: Ars Magica - Definitive Edition
    /// (Core Rules).md:15791-15794 (the (Form) is
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

    /// A grog combat line with a weapon and a shield combined (Ars Magica - Definitive Edition (Core Rules).md:16658-16670,
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
        // The shield still weighs on Encumbrance in both lines (Ars Magica - Definitive Edition (Core Rules).md:17107).
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
            equipped: true,
            specialization_applies: false,
        }];
        let lines = combat_totals(&e, &rs);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].shields.is_empty());
        // Defense = Qik 1 + Ability 4 + WpnDef 1 = 6, no shield anywhere.
        assert_eq!(lines[0].defense, 6);
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
            equipped: true,
            specialization_applies: false,
        };
        e.equipment = vec![
            EquipmentSlot {
                item: Id::new("weapon.long_sword"),
                equipped: true,
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
    /// Encumbrance (Ars Magica - Definitive Edition (Core Rules).md:7494, :17107).
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
    /// Initiative (which do not use the Ability) are unchanged (Ars Magica - Definitive Edition (Core Rules).md:7122, :7139).
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
    /// combat-gear Load ≥ half of total Load exempts Attack/Defense (Ars Magica - Definitive Edition (Core Rules).md:17105).
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:17105, :16658).
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

    /// Soak with Tough (+3) and a Bronze cord, plus worn armor (Ars Magica - Definitive Edition (Core Rules).md:16667,
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:17103-17123).
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

    /// Wound ranges for Size 0 and Size +1 (Ars Magica - Definitive Edition (Core Rules).md:17167-17180).
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

    /// Size +1 widens every band by the unit growth (Ars Magica - Definitive Edition (Core Rules).md:17167-17180).
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:3751-3754).
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
    /// from `effective.rs`, not reimplemented (Ars Magica - Definitive Edition (Core Rules).md:16617, :16464-16475).
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
    /// every cell (Ars Magica - Definitive Edition (Core Rules).md:4151-4154).
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

    /// Weak Enchanter (GD3, round 2): "Halve your Lab Total whenever you
    /// create or investigate an enchanted item. If you have a Deficiency that
    /// counts as part of the Lab Total, apply the Deficiency first and then
    /// halve the remaining total" (Ars Magica - Definitive Edition (Core
    /// Rules).md:7060-7063). `HalvableTotal::LabEnchanting` was asserted by
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
    /// contribution, leaving resistance from Parma alone (Ars Magica - Definitive Edition (Core Rules).md:6346-6349).
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
    /// listed. Source: Ars Magica - Definitive Edition (Core Rules).md:6815-6826.
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
    /// subject as its detail and the bonus as its amount (Ars Magica - Definitive Edition (Core Rules).md:3362-3367).
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

    /// `Effect::AbilityRollMod` (and `Effect::MagicalFocus`) carry a `text`-domain
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
        let loaded = crate::load_entity_migrating(&serde_json::to_string(&e).unwrap())
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
    /// five" (Ars Magica - Definitive Edition (Core Rules).md:7084-7086, not
    /// `:7060-7063` — that range is Weak Enchanter, a different Flaw
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
    /// (`formulaic_casting_score`, Ars Magica - Definitive Edition (Core Rules).md:5913-5915).
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
    /// penalties (Ars Magica - Definitive Edition (Core Rules).md:17127-17129).
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
