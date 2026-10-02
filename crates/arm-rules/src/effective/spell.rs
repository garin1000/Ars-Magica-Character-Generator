//! Spell levels and Spell Mastery: the levels budget (base + Skilled/Weak
//! Parens + post-Gauntlet life-stage levels), per-Technique/Form level caps,
//! and the Spell-Mastery XP pool/floor/Affinity chain (Flawless Magic). Split
//! out of `effective.rs` (Viktor's V4 architecture finding — 93 free functions
//! across 7 unrelated domains in one file); pure code motion, no behavior
//! change.

use super::*;
use crate::art::ArtType;
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
        // D40/D2: a life-stage XP replacement, not a spell-levels contribution.
        | Effect::ReplacesLifeStageXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        // F-63/C5c: a floor grant, not a spell-levels contribution.
        | Effect::AbilityScoreGrantParam { .. }
        | Effect::GeneralXp { .. }
        | Effect::LaterLifeXpRate { .. }
        | Effect::SuppressesLaterLifeXpRate
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
        | Effect::RelicTrueFaith { .. }
        | Effect::WarpingGrant { .. }
        // D69/X7b-e: a Warping grant, not a spell-levels/general-XP contribution.
        | Effect::WarpingGrantParam { .. }
        | Effect::SizeDelta { .. }
        | Effect::CharacteristicScoreDelta { .. }
        // D69/X7b-e: a Characteristic buy-cap shift, not a contribution here.
        | Effect::CharacteristicMax { .. }
        | Effect::GroupAffinityCost { .. }
        | Effect::GrantsReputation { .. }
        // B3/D23/F-542: a narrative Personality-Trait grant, consumed only
        // by `ItemPredicate::GrantsPersonalityTrait`'s derivation.
        | Effect::GrantsPersonalityTrait
        // D69/X7b-e: creation-legality constraints on Personality Traits, not
        // a contribution here.
        | Effect::PersonalityTraitRange { .. }
        | Effect::RequiresPersonalityTraitPair { .. }
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
        | Effect::AbilityRollModParam { .. }
        | Effect::ElementalMagic { .. }
        | Effect::ForbidsAbilitySpecialties
        | Effect::ForbidsRitualCasting
        | Effect::WaivesAbilityAgeCap
        // Not a spell-levels contribution — training is a creation-legality
        // fact, not a levels grant.
        | Effect::ConfersHermeticTraining
        | Effect::ConfersHermeticTrainingIf { .. }
        // D3: the 8×years spell-levels term is folded via
        // `LifeStageBudget.truncated_training_spell_levels`
        // (`life_stage::LifeStageRules::budget`), a SEPARATE selector —
        // adding it here too would double-count.
        | Effect::TruncatedApprenticeshipXp { .. }
        // B1/D21: category/ability prohibitions — no spell-levels
        // contribution, same reasoning as the markers above.
        | Effect::ForbidsAbilityCategory { .. }
        | Effect::ForbidsAbilityCategoryParam { .. }
        | Effect::ForbidsItemCategory { .. }
        | Effect::ForbidsAbilities { .. }
        // X6a/e6: folded only by `ability_age_cap`, no contribution here.
        | Effect::AbilityScoreCapOverrideParam { .. }
        | Effect::AbilityScoreCapAllExcept { .. }
        // D69/X7b-e: a surfaced-only roll penalty, not a contribution here.
        | Effect::DecrepitudeScaledRollMod { .. }
        // D68.11: an id-less category-cap count, not a spell-levels contribution.
        | Effect::GrantsCategoryCount { .. } => None,
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
        // D40/D2: folded in `effective/xp.rs`, not here, or it would
        // double-count exactly as this file's own `LaterLifeXpRate` comment
        // warns against.
        | Effect::ReplacesLifeStageXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        // F-63/C5c: a floor grant, not a general-XP contribution.
        | Effect::AbilityScoreGrantParam { .. }
        | Effect::SpellLevels { .. }
        // The later-life RATE is not a pool bonus: it multiplies out into the
        // life-stage budget (see `life_stage::LifeStageRules::later_life_budget`),
        // which then becomes the general pool. Adding it here would double-count.
        | Effect::LaterLifeXpRate { .. }
        | Effect::SuppressesLaterLifeXpRate
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
        | Effect::RelicTrueFaith { .. }
        | Effect::WarpingGrant { .. }
        // D69/X7b-e: a Warping grant, not a spell-levels/general-XP contribution.
        | Effect::WarpingGrantParam { .. }
        | Effect::SizeDelta { .. }
        | Effect::CharacteristicScoreDelta { .. }
        // D69/X7b-e: a Characteristic buy-cap shift, not a contribution here.
        | Effect::CharacteristicMax { .. }
        | Effect::GroupAffinityCost { .. }
        | Effect::GrantsReputation { .. }
        // B3/D23/F-542: a narrative Personality-Trait grant, consumed only
        // by `ItemPredicate::GrantsPersonalityTrait`'s derivation.
        | Effect::GrantsPersonalityTrait
        // D69/X7b-e: creation-legality constraints on Personality Traits, not
        // a contribution here.
        | Effect::PersonalityTraitRange { .. }
        | Effect::RequiresPersonalityTraitPair { .. }
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
        | Effect::AbilityRollModParam { .. }
        | Effect::ElementalMagic { .. }
        | Effect::ForbidsAbilitySpecialties
        | Effect::ForbidsRitualCasting
        | Effect::WaivesAbilityAgeCap
        // Not a general-XP contribution — the apprenticeship *shape* this
        // confers is folded in `effective/xp.rs`, not here, or it would
        // double-count exactly as this file's own `LaterLifeXpRate` comment
        // warns against.
        | Effect::ConfersHermeticTraining
        | Effect::ConfersHermeticTrainingIf { .. }
        // D3: the 16×years XP term is folded via `general_pool_and_bonus`
        // (`effective/xp.rs`), not this per-effect fold — same reasoning as
        // `ConfersHermeticTraining` above.
        | Effect::TruncatedApprenticeshipXp { .. }
        // B1/D21: category/ability prohibitions — no general-XP
        // contribution, same reasoning as the markers above.
        | Effect::ForbidsAbilityCategory { .. }
        | Effect::ForbidsAbilityCategoryParam { .. }
        | Effect::ForbidsItemCategory { .. }
        | Effect::ForbidsAbilities { .. }
        // X6a/e6: folded only by `ability_age_cap`, no contribution here.
        | Effect::AbilityScoreCapOverrideParam { .. }
        | Effect::AbilityScoreCapAllExcept { .. }
        // D69/X7b-e: a surfaced-only roll penalty, not a contribution here.
        | Effect::DecrepitudeScaledRollMod { .. }
        // D68.11: an id-less category-cap count, not a general-XP contribution.
        | Effect::GrantsCategoryCount { .. } => None,
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
        .map_or(0, |budget| {
            // D3/D56: an Abandoned Apprentice's truncated block also grants
            // spell levels (8×years) — mutually exclusive with a real
            // magus's post-Gauntlet split in practice, so summing is safe.
            budget
                .post_gauntlet_spell_levels
                .saturating_add(budget.truncated_training_spell_levels)
        })
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
            // D1 deliberately ignores `scope`/`suppressed_when`: those resolve
            // D4's in-play conditions, and D1 stays the flat, condition-free
            // ceiling regardless of them (this function's own doc comment).
            if let Effect::LabTotalMod { amount, .. } = effect {
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

/// Whether the entity holds a Magical Focus ([`Effect::MagicalFocus`]) at all —
/// read here (not via `derived.rs::InPlayMods::has_focus`, which is private to
/// that module) so [`spell_caps`] knows whether a within-focus figure is worth
/// computing for the picker at all. Mirrors [`has_short_ranged_magic`]'s shape.
///
/// `pub(crate)` (D81.17, `docs/vf-audit/decisions.md`): also read by
/// `validation/magus.rs::validate_spell_focus_marker_without_focus`, so the
/// warning for a stale `within_focus` marker can never disagree with the SAME
/// predicate that already neutralizes the marker's effect on the cap and the
/// Casting Total, right below.
pub(crate) fn has_magical_focus(entity: &Entity, ruleset: &Ruleset) -> bool {
    selections_for_effects(entity, ruleset)
        .iter()
        .any(|selection| {
            ruleset
                .point_items
                .get(&selection.item_ref)
                .is_some_and(|item| {
                    item.effects
                        .iter()
                        .any(|effect| matches!(effect, Effect::MagicalFocus { .. }))
                })
        })
}

/// The maximum level a magus may learn of a spell of the given Technique/Form:
/// the sum of Technique, Form, Intelligence, Magic Theory and 3 (ArMDE:2465),
/// using effective Art/Ability scores, **folded with `requisites`** the same
/// way the Casting Total is (ArMDE:12309-12313, X11b — see below), **doubled**
/// by the lowest folded score when `within_focus` is set (ArMDE:4403),
/// **halved** if either Art or any requisite is deficient, plus the flat
/// [`lab_total_mod`] term (D1, `docs/vf-audit/decisions.md`), and **halved
/// again** if `range_beyond_touch` is set and the character holds Short-Ranged
/// Magic (D28). Returns an `i64` (small or negative for a beginning magus).
/// Single source of truth: both the validation cap and the UI-surfaced cap
/// read this, so the two can never diverge.
///
/// **Requisite folding (X11b, D81.5).** `ArMDE:2465`'s own second sentence —
/// "If the spell has requisites (see page 311), they apply to this total as
/// well" — and its closing sentence, which calls the cap itself "the
/// appropriate Lab Total", bring in exactly the same lesser-of-requisite-and-
/// primary rule the Casting Total already folds (`b5ee82a`, X11):
/// `derived/casting.rs::fold_requisite` is reused here verbatim, not
/// duplicated, so a Puissant Art bonus (ArMDE:4820), several requisites of one
/// class folding to their group's lowest (ArMDE:12311), and the Elemental
/// Magic exception (ArMDE:3737, "you use the primary Form to calculate totals,
/// even if the requisite is lower" — generic "totals" wording, so it governs
/// this Lab-Total-shaped cap too) all behave identically on both totals. A
/// requisite Art that is itself Deficient halves the cap even when it does not
/// numerically bind the fold (ArMDE:12311's closing sentence) — `deficient`
/// below checks `requisites` for exactly this reason, reading the same
/// [`deficient_arts`] fold the Casting Total's `InPlayMods::deficient` does, so
/// the two can never disagree about which Arts are deficient. The grid
/// (`spell_level_caps`) has no specific spell at a cell (it may host several),
/// so it passes `requisites: &[]` — a no-op fold, exactly like
/// `derived/casting.rs::casting_totals` does for the same reason.
///
/// **Magical Focus doubling (ArMDE:4403, X11b).** "If a spell has requisites,
/// the lowest applicable score may be one of the requisites, rather than one
/// of the primary Arts" — since `within_focus` adds the lower of the two
/// *already-folded* scores, a requisite that won the fold is automatically
/// eligible, with no separate case needed (confirmed by hand against the
/// passage's own worked example and by
/// `crates/arm-rules/tests/requisite_level_cap.rs`'s focus test). `within_focus`
/// is the caller's own claim (`SpellSelection::within_focus`, X10c) — the
/// engine cannot match a free-text focus theme to a spell (MAG8), so the
/// player decides and the validator reads that choice back
/// (`validation/magus.rs::validate_spell_level_cap`).
///
/// `range_beyond_touch` is a fact about the **spell being asked about** (its
/// own Range), not the character. `validation/magus.rs::validate_spell_level_cap`
/// derives it from a real spell's `Option<SpellRange>` via the free function
/// [`range_beyond_touch`]; [`spell_level_caps`] instead iterates both range
/// classes directly (`false`, `true`) to synthesize the two surfaced rows.
///
/// **Order of operations**, from the passages: the flat D1 term and the
/// within-focus double both sum into `base` first (they are part of what the
/// Lab Total *is* — `ArMDE:2465`'s closing sentence, and `ArMDE:4403`'s own
/// worked example adds the doubled Art alongside the rest before anything else
/// applies), then the two conditional halvings apply. The halving is not a
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
/// reads the same [`deficient_arts`] fold Deficient-Art halving always has,
/// through the same [`arts_deficient`] predicate [`crate::derived::InPlayMods::deficient`]
/// applies to its own cached set, so the creation-time cap and the in-play Lab
/// Totals can never disagree about which Arts are deficient.
// Source: ArMDE:2465, :12309-12313, :3737, :4403, :4820, :5911, :5915, :547, :6739
pub fn spell_level_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    technique: &Id,
    form: &Id,
    requisites: &[Id],
    range_beyond_touch: bool,
    within_focus: bool,
) -> i64 {
    // Read before the Art *scores* shadow `technique`/`form` with their totals.
    let deficiencies = deficient_arts(entity, ruleset);
    let deficient = arts_deficient(&deficiencies, technique, form, requisites);
    let elemental_forms = elemental_magic_forms(entity, ruleset);
    let tech = i64::from(crate::derived::casting::fold_requisite(
        entity,
        ruleset,
        ArtType::Technique,
        technique,
        effective_art_score(entity, ruleset, technique),
        requisites,
        None,
    ));
    let fo = i64::from(crate::derived::casting::fold_requisite(
        entity,
        ruleset,
        ArtType::Form,
        form,
        effective_art_score(entity, ruleset, form),
        requisites,
        elemental_forms.as_ref(),
    ));
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
    let mut base = tech + fo + int + magic_theory + 3 + i64::from(lab_total_mod(entity, ruleset));
    // A stale marker (the Magical Focus Virtue removed after the spell was
    // marked `within_focus`) must not double the cap — the doubling applies
    // only while the entity actually holds a Magical Focus right now. Saves
    // store choices, so `within_focus` itself is never scrubbed; only its
    // effect here is gated.
    if within_focus && has_magical_focus(entity, ruleset) {
        base += tech.min(fo);
    }
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
                    // No specific spell at this grid cell (it may host several,
                    // each with different or no requisites, and none of them
                    // specifically marked within-focus) — a no-op fold, same
                    // reasoning as `derived/casting.rs::casting_totals`'s own
                    // `requisites: &[]`.
                    cap: spell_level_cap(
                        entity,
                        ruleset,
                        technique,
                        form,
                        &[],
                        range_beyond_touch,
                        false,
                    ),
                });
            }
        }
    }
    caps
}

/// A per-catalogue-spell level cap (X11b, D81.5): unlike [`SpellLevelCap`]'s
/// Te/Fo/range-keyed grid, this folds the spell's own `requisites`, so two
/// spells sharing a Te/Fo pair but different requisites can genuinely report
/// different caps — the same reason `derived/casting.rs::spell_casting_total`
/// exists beside `casting_totals`. Serializes as `{ "spell": "<id>", "cap": N,
/// "within_focus_cap": N | null }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellCap {
    /// The catalogue spell's id.
    pub spell: Id,
    /// The maximum learnable level for this spell, requisites folded, no
    /// Magical Focus doubling (may be negative for a beginning magus).
    pub cap: i64,
    /// The same cap WITH the Magical Focus doubling (ArMDE:4403) applied,
    /// present only when the entity holds a Magical Focus at all — the
    /// picker's "add within focus" action offers itself only then. `None` for
    /// an entity with no Magical Focus Virtue, same shape as
    /// [`crate::derived::CastingTotal::within_focus`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_focus_cap: Option<i64>,
}

/// [`spell_level_cap`] for every spell in the catalogue, folding each spell's
/// own `requisites` and, only when the entity holds a Magical Focus, also
/// computing the focus-doubled figure. The UI picker greys a spell by
/// [`SpellCap::cap`] and, when only [`SpellCap::within_focus_cap`] admits the
/// spell's level, offers an "add within focus" action that adds it already
/// marked (`SpellSelection::within_focus = true`) — D81.5: "The engine cannot
/// match a spell to a free-text focus, so the player decides."
pub fn spell_caps(entity: &Entity, ruleset: &Ruleset) -> Vec<SpellCap> {
    let has_focus = has_magical_focus(entity, ruleset);
    ruleset
        .spells()
        .map(|spell| {
            let range_beyond_touch = spell.range.is_some_and(range_beyond_touch);
            let cap = spell_level_cap(
                entity,
                ruleset,
                &spell.technique,
                &spell.form,
                &spell.requisites,
                range_beyond_touch,
                false,
            );
            let within_focus_cap = has_focus.then(|| {
                spell_level_cap(
                    entity,
                    ruleset,
                    &spell.technique,
                    &spell.form,
                    &spell.requisites,
                    range_beyond_touch,
                    true,
                )
            });
            SpellCap {
                spell: spell.id.clone(),
                cap,
                within_focus_cap,
            }
        })
        .collect()
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
                // D40/D2: not a Spell Mastery Affinity — grants no advancement
                // multiplier.
                | Effect::ReplacesLifeStageXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                // F-63/C5c: a floor grant, no advancement Affinity.
                | Effect::AbilityScoreGrantParam { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::SuppressesLaterLifeXpRate
                | Effect::AbilityAuthorization { .. }
                | Effect::AbilityBonusGated { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
                | Effect::GrantsSelection { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::MasterpieceItem
                | Effect::TrueFaithGrant { .. }
                | Effect::RelicTrueFaith { .. }
                | Effect::WarpingGrant { .. }
                | Effect::WarpingGrantParam { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::CharacteristicMax { .. }
                | Effect::GroupAffinityCost { .. }
                | Effect::GrantsReputation { .. }
                // B3/D23/F-542: consumed only by
                // `ItemPredicate::GrantsPersonalityTrait`'s derivation.
                | Effect::GrantsPersonalityTrait
                | Effect::PersonalityTraitRange { .. }
                | Effect::RequiresPersonalityTraitPair { .. }
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
                | Effect::AbilityRollModParam { .. }
                | Effect::ElementalMagic { .. }
                | Effect::ForbidsAbilitySpecialties
                | Effect::ForbidsRitualCasting
                | Effect::WaivesAbilityAgeCap
                // Not a Spell Mastery Affinity — grants no advancement
                // multiplier.
                | Effect::ConfersHermeticTraining
                | Effect::ConfersHermeticTrainingIf { .. }
                // D3: a general-XP/life-stage-years grant, no advancement
                // multiplier either.
                | Effect::TruncatedApprenticeshipXp { .. }
                // B1/D21: category/ability prohibitions — no advancement
                // multiplier either.
                | Effect::ForbidsAbilityCategory { .. }
                | Effect::ForbidsAbilityCategoryParam { .. }
                | Effect::ForbidsItemCategory { .. }
                | Effect::ForbidsAbilities { .. }
                | Effect::AbilityScoreCapOverrideParam { .. }
                | Effect::AbilityScoreCapAllExcept { .. }
                | Effect::DecrepitudeScaledRollMod { .. }
                // D68.11: an id-less category-cap count, not a Spell Mastery
                // Affinity.
                | Effect::GrantsCategoryCount { .. } => None,
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

// --- Incompatible Arts (D81.8) ----------------------------------------------

/// The set of `(Technique, Form)` pairs barred by every held copy of an item
/// declaring [`crate::types::PointItem::unordered_param_groups`] — today only
/// `flaw.incompatible_arts`, but driven entirely by that data field, not by
/// this item's id (ArMDE:6290-6292, "unable to use two combinations of
/// Techniques and Forms"). Each group's named values are resolved against
/// THIS selection's own `params` and placed by which key's
/// [`ParameterDomain`] is `Technique` vs `Form` — not by position in the
/// group — so a group declares its two keys in either order with no effect on
/// which pair results.
///
/// Grant-aware: `selections` is expected to be the folded bought-plus-granted
/// list ([`crate::effective::selections_for_effects`]), matching every other
/// reader in this module.
pub(crate) fn barred_combinations(
    selections: &[Selection],
    ruleset: &Ruleset,
) -> BTreeSet<(Id, Id)> {
    let mut barred = BTreeSet::new();
    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for group in &item.unordered_param_groups {
            let mut technique = None;
            let mut form = None;
            for key in group {
                let Some(def) = item.parameters.iter().find(|p| &p.key == key) else {
                    continue;
                };
                let Some(value) = selection
                    .params
                    .get(key)
                    .and_then(SelectionParamValue::as_single)
                else {
                    continue;
                };
                match def.domain {
                    ParameterDomain::Technique => technique = Some(value.clone()),
                    ParameterDomain::Form => form = Some(value.clone()),
                    _ => {}
                }
            }
            if let (Some(t), Some(f)) = (technique, form) {
                barred.insert((t, f));
            }
        }
    }
    barred
}

/// Whether `spell` draws on any pair in `barred` — either directly, as its
/// primary Technique+Form, or "even if one or both are requisites"
/// (ArMDE:6292): the cross product of `{primary technique} ∪ {Technique-class
/// requisites}` × `{primary form} ∪ {Form-class requisites}`
/// (ArMDE:12309-12311, "Sometimes a spell has a requisite for both its
/// Technique and Form" — the same reading `derived/casting.rs::fold_requisite`
/// already applies to the numeric Casting Total).
pub(crate) fn spell_touches_barred_combination(
    spell: &crate::spell::Spell,
    ruleset: &Ruleset,
    barred: &BTreeSet<(Id, Id)>,
) -> bool {
    if barred.is_empty() {
        return false;
    }
    let mut techniques = vec![&spell.technique];
    let mut forms = vec![&spell.form];
    for req in &spell.requisites {
        match ruleset.art(req).map(|a| a.art_type) {
            Some(ArtType::Technique) => techniques.push(req),
            Some(ArtType::Form) => forms.push(req),
            None => {}
        }
    }
    techniques.iter().any(|t| {
        forms
            .iter()
            .any(|f| barred.contains(&((*t).clone(), (*f).clone())))
    })
}
