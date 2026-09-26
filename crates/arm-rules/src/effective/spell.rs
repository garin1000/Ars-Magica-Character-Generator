//! Spell levels and Spell Mastery: the levels budget (base + Skilled/Weak
//! Parens + post-Gauntlet life-stage levels), per-Technique/Form level caps,
//! and the Spell-Mastery XP pool/floor/Affinity chain (Flawless Magic). Split
//! out of `effective.rs` (Viktor's V4 architecture finding — 93 free functions
//! across 7 unrelated domains in one file); pure code motion, no behavior
//! change.

use super::*;
use crate::spell::SpellRange;

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
        | Effect::CharacteristicScoreDeltaParam { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::ScaledRestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
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
        | Effect::MasterpieceItem
        | Effect::TrueFaithGrant { .. }
        | Effect::WarpingGrant { .. }
        | Effect::SizeDelta { .. }
        | Effect::CharacteristicScoreDelta { .. }
        | Effect::GroupAffinityCost { .. }
        | Effect::GrantsReputation { .. }
        | Effect::MightGrant { .. }
        | Effect::PowerLevels { .. }
        | Effect::FocusPoints { .. }
        | Effect::MagicalFocus { .. }
        | Effect::CastingTotalMod { .. }
        | Effect::LabTotalMod { .. }
        | Effect::HalvesSpellCapBeyondTouch
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
        | Effect::ElementalMagic { .. }
        | Effect::ForbidsAbilitySpecialties
        | Effect::ForbidsRitualCasting
        | Effect::WaivesAbilityAgeCap
        // Not a spell-levels contribution — training is a creation-legality
        // fact, not a levels grant.
        | Effect::ConfersHermeticTraining => None,
    })
}

/// Sums the [`Effect::GeneralXp`] amounts across the entity's selections (may be
/// negative; Skilled Parens +60, Weak Parens −60).
///
/// `pub(crate)` (not just module-private) because
/// [`xp_allocation`](crate::effective::xp_allocation) (the `xp` domain) folds
/// this into the general pool's size.
pub(crate) fn general_xp_bonus(entity: &Entity, ruleset: &Ruleset) -> i64 {
    sum_signed_effect(entity, ruleset, |e| match e {
        Effect::GeneralXp { amount } => Some(*amount),
        // Exhaustive so adding an Effect variant is a compile error here, not a
        // silently-ignored contribution to the general XP pool.
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicScoreDeltaParam { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::ScaledRestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        | Effect::SpellLevels { .. }
        // The later-life RATE is not a pool bonus: it multiplies out into the
        // life-stage budget (see `life_stage::LifeStageRules::later_life_budget`),
        // which then becomes the general pool. Adding it here would double-count.
        | Effect::LaterLifeXpRate { .. }
        | Effect::AbilityAuthorization { .. }
        | Effect::AbilityBonusGated { .. }
        | Effect::LocalityAbilityCapFraction { .. }
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
        | Effect::FocusPoints { .. }
        | Effect::MagicalFocus { .. }
        | Effect::CastingTotalMod { .. }
        | Effect::LabTotalMod { .. }
        | Effect::HalvesSpellCapBeyondTouch
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
        | Effect::ElementalMagic { .. }
        | Effect::ForbidsAbilitySpecialties
        | Effect::ForbidsRitualCasting
        | Effect::WaivesAbilityAgeCap
        // Not a general-XP contribution — the apprenticeship *shape* this
        // confers is folded in `effective/xp.rs`, not here, or it would
        // double-count exactly as this file's own `LaterLifeXpRate` comment
        // warns against.
        | Effect::ConfersHermeticTraining => None,
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
// Source: ArMDE:2215-2216, :2435
pub fn spell_levels_base(entity: &Entity, profile: Option<&EntityTypeProfile>) -> u32 {
    entity
        .spell_levels_override
        .unwrap_or_else(|| profile.map(|p| p.spell_levels).unwrap_or(0))
}

/// The levels of spells a magus took out of its years past the Gauntlet — the
/// player's chosen slice of "30 points per year", where "Each point can be an
/// experience point in an Art or Ability or **one level of spell**" (`ArMDE:2471`).
///
/// 0 for a character with no life-stage plan, and for a ruleset shipping no
/// `post_apprenticeship` block — the rate is data, so with no block there is
/// nothing to grant.
// Source: ArMDE:2216, :2471.
pub fn life_stage_spell_levels(entity: &Entity, ruleset: &Ruleset) -> u32 {
    ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset))
        .map_or(0, |budget| budget.post_gauntlet_spell_levels)
}

/// The magus's effective spell-levels budget: the base ([`spell_levels_base`])
/// plus any [`Effect::SpellLevels`] modifiers and the levels its post-Gauntlet
/// years bought ([`life_stage_spell_levels`]), clamped at 0.
///
/// The post-Gauntlet term is **additive, not a second budget.** Apprenticeship's
/// "120 levels of spells" (`ArMDE:2435`) are the type profile's `spell_levels` and are
/// what `base` selects; these are the player's chosen slice of the fungible "30
/// points per year" (`ArMDE:2471`), which is also why `post_gauntlet_xp` and
/// `post_gauntlet_spell_levels` always sum to `post_gauntlet_points`.
///
/// Folded in **here**, in the one selector both `validate_spells` and the
/// `EffectiveScores` payload call, so the `over_spell_levels` finding and the
/// spell-levels bar can never disagree about what the budget is.
///
/// [`Entity::spell_levels_override`] still replaces the *profile base* only, and
/// the post-Gauntlet levels stay on top of it. Deliberate: the override is the flat
/// flow's escape hatch, and it is not made exclusive with a life-stage plan the way
/// [`Entity::xp_pool`] is.
// Source: ArMDE:2216, :2435, :2471.
pub fn spell_levels_budget(base: u32, entity: &Entity, ruleset: &Ruleset) -> u32 {
    clamp_to_u32(
        i64::from(base)
            + spell_levels_bonus(entity, ruleset)
            + i64::from(life_stage_spell_levels(entity, ruleset)),
    )
}

/// The flat sum of every [`Effect::LabTotalMod`] amount (Inventive Genius +3,
/// Creative Block −3, …) across the entity's selections + grants — the same
/// fold `derived::lab_totals`'s `lab_mod` addend sums, reused here (not
/// duplicated) so the in-play Lab-Total grid and [`spell_level_cap`]'s D1 term
/// can never disagree about what the flat sum is.
///
/// D1 (`docs/vf-audit/decisions.md`): every one of the nine carriers is
/// individually *conditional* in the book (Inventive Genius only "if you are
/// not using a Laboratory Text or being taught"; Potent Magic only within its
/// focus; …), and D4 resolves those conditions for the in-play Lab Total. This
/// function is consumed **only** by [`spell_level_cap`], which deliberately
/// ignores every condition and always adds the flat sum — it is only a ceiling
/// on which spells may be *chosen*, never a number printed as a play result, so
/// the generous condition-free reading is acceptable there and nowhere else.
pub(crate) fn lab_total_mod(entity: &Entity, ruleset: &Ruleset) -> i32 {
    let mut total = 0i32;
    for selection in selections_for_effects(entity, ruleset).iter() {
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

/// Whether a spell's Range makes it subject to Short-Ranged Magic's Lab-Total
/// halving (D28, `docs/vf-audit/decisions.md`): Eye, Voice, Sight, and Arcane
/// Connection — **not** Personal or Touch.
///
/// Matched **by name**, deliberately never derived from [`SpellRange`]'s
/// difficulty ordering: Eye sits at the *same* RDT difficulty as Touch, and the
/// book calls it out explicitly for exactly that reason — "a range greater than
/// Touch, **including Eye**" would be redundant to state if Eye already fell out
/// of "greater than" by magnitude. The whitelist is the honest reading; a
/// `range > SpellRange::Touch` comparison would happen to agree today only
/// because of `SpellRange`'s declaration order, and that is not something this
/// rule may rely on.
///
/// Source: ArMDE:6739.
pub(crate) fn range_beyond_touch(range: SpellRange) -> bool {
    matches!(
        range,
        SpellRange::Eye | SpellRange::Voice | SpellRange::Sight | SpellRange::ArcaneConnection
    )
}

/// Whether the entity holds [`Effect::HalvesSpellCapBeyondTouch`] (Short-Ranged
/// Magic) — folded separately from [`lab_total_mod`] because this halving is
/// conditional on the *spell being evaluated*, not a flat addend every cap
/// reads regardless of which spell is asked about.
fn has_short_ranged_magic(entity: &Entity, ruleset: &Ruleset) -> bool {
    selections_for_effects(entity, ruleset)
        .iter()
        .any(|selection| {
            ruleset
                .point_items
                .get(&selection.item_ref)
                .is_some_and(|item| item.effects.contains(&Effect::HalvesSpellCapBeyondTouch))
        })
}

/// The maximum level a magus may learn of a spell of the given Technique/Form:
/// the sum of Technique, Form, Intelligence, Magic Theory and 3 (ArMDE:2465),
/// using effective Art/Ability scores, **halved** if either Art is deficient,
/// plus the flat [`lab_total_mod`] term (D1, `docs/vf-audit/decisions.md`), and
/// **halved again** if `range_beyond_touch` is set and the character holds
/// Short-Ranged Magic (D28). Returns an `i64` (small or negative for a
/// beginning magus). Requisite-Art reduction is a lab-total nuance out of
/// scope. Single source of truth: both the validation cap and the UI-surfaced
/// cap read this, so the two can never diverge.
///
/// `range_beyond_touch` is a fact about the **spell being asked about** (its
/// own Range), not the character. `validation/magus.rs::validate_spell_level_cap`
/// derives it from a real spell's `Option<SpellRange>` via the free function
/// [`range_beyond_touch`]; [`spell_level_caps`] instead iterates both range
/// classes directly (`false`, `true`) to synthesize the two surfaced rows.
///
/// **Order of operations**, from the passages: the flat D1 term is summed into
/// `base` first (it is part of what the Lab Total *is*, `ArMDE:2465`'s closing
/// sentence), then the two conditional halvings apply. The halving is not a
/// separate rule but the same sentence: `ArMDE:2465` ends "This is the
/// appropriate Lab Total, assuming an aura modifier of +3, and thus any Virtues
/// and Flaws your character has apply to this total if they would apply to a
/// Lab Total in play", and a Deficiency halves every Lab Total its Art is added
/// to (`ArMDE:5911, :5915`) while Short-Ranged Magic halves it "when designing
/// an effect or spell that has a range greater than Touch, including Eye"
/// (`ArMDE:6739`). Nothing in either passage orders the two conditional
/// halvings relative to each other, and — like the Deficient/Difficult-Longevity
/// stack in `derived/lab.rs::creo_corpus_lab_total` — the order between them is
/// provably immaterial: both are a plain `div_euclid(_, 2)`, and floor division
/// by 2 twice equals floor division by 4 regardless of which comes first. This
/// reads the same `effective/art.rs::deficient_arts` fold Deficient-Art halving
/// always has, so the creation-time cap and the in-play Lab Totals can never
/// disagree about which Arts are deficient.
// Source: ArMDE:2465, :5911, :5915, :547, :6739
pub fn spell_level_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    technique: &Id,
    form: &Id,
    range_beyond_touch: bool,
) -> i64 {
    // Read before the Art *scores* shadow `technique`/`form` with their totals.
    let deficiencies = deficient_arts(entity, ruleset);
    let deficient = deficiencies.contains(technique) || deficiencies.contains(form);
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
    let base = tech + form + int + magic_theory + 3 + i64::from(lab_total_mod(entity, ruleset));
    // Floor, not truncate, for both halvings below. No halving rule names a
    // rounding direction, so the rulebook default governs — "if it does not,
    // round down" (ArMDE:547) — and this cap is routinely negative for a
    // beginning magus, where `/ 2` would round -1 up to 0 and hand out a free
    // level-0 spell. Same `div_euclid` the in-play totals halve through
    // (`derived.rs::halve`).
    let mut cap = if deficient { base.div_euclid(2) } else { base };
    if range_beyond_touch && has_short_ranged_magic(entity, ruleset) {
        cap = cap.div_euclid(2);
    }
    cap
}

/// A per-Technique/Form/range-class spell-level cap, surfaced to the frontend
/// so the spell picker can grey a spell whose level exceeds the magus's cap
/// without recomputing the derivation in JS. Serializes as
/// `{ "technique": "<id>", "form": "<id>", "range_beyond_touch": bool, "cap": N }`.
///
/// D28 (`docs/vf-audit/decisions.md`): the cap no longer depends on Te/Fo
/// alone once Short-Ranged Magic is in play, so the key gains
/// `range_beyond_touch` alongside the Te/Fo pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellLevelCap {
    /// The Technique-class Art id (e.g. `art.creo`).
    pub technique: Id,
    /// The Form-class Art id (e.g. `art.ignem`).
    pub form: Id,
    /// Whether this row is the beyond-Touch cap (Eye, Voice, Sight, Arcane
    /// Connection) or the ordinary Personal/Touch cap for this Te/Fo pair.
    pub range_beyond_touch: bool,
    /// The maximum learnable level for this Te/Fo/range-class combination (may
    /// be negative for a beginning magus).
    pub cap: i64,
}

/// The [`spell_level_cap`] for every Technique × Form combination in the Art
/// catalogue, crossed with both range classes (D28), sorted canonically by
/// `(technique, form, range_beyond_touch)`. The picker keys these by
/// `(technique, form, range_beyond_touch)` — the last computed from a candidate
/// spell's own Range via [`range_beyond_touch`] — to look up its cap. Two
/// entries per Te/Fo combo (a spell's cap depends on its Te/Fo and whether its
/// Range is beyond Touch, never its level).
pub fn spell_level_caps(entity: &Entity, ruleset: &Ruleset) -> Vec<SpellLevelCap> {
    // `art_ids_of` guarantees the sort the canonical (technique, form) order needs.
    let techniques = ruleset.art_ids_of(crate::art::ArtType::Technique);
    let forms = ruleset.art_ids_of(crate::art::ArtType::Form);
    let mut caps = Vec::with_capacity(techniques.len() * forms.len() * 2);
    for technique in &techniques {
        for form in &forms {
            for range_beyond_touch in [false, true] {
                caps.push(SpellLevelCap {
                    technique: technique.clone(),
                    form: form.clone(),
                    range_beyond_touch,
                    cap: spell_level_cap(entity, ruleset, technique, form, range_beyond_touch),
                });
            }
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
/// spent only on per-spell Spell Mastery Abilities. Source: ArMDE:4471-4474.
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
/// Source: ArMDE:3887-3889.
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
/// Source: ArMDE:3889.
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
                | Effect::CharacteristicScoreDeltaParam { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::ScaledRestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::AbilityBonusGated { .. }
                | Effect::LocalityAbilityCapFraction { .. }
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
                | Effect::FocusPoints { .. }
                | Effect::MagicalFocus { .. }
                | Effect::CastingTotalMod { .. }
                | Effect::LabTotalMod { .. }
                | Effect::HalvesSpellCapBeyondTouch
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
                | Effect::ElementalMagic { .. }
                | Effect::ForbidsAbilitySpecialties
                | Effect::ForbidsRitualCasting
                | Effect::WaivesAbilityAgeCap
                // Not a Spell Mastery Affinity — grants no advancement
                // multiplier.
                | Effect::ConfersHermeticTraining => None,
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
