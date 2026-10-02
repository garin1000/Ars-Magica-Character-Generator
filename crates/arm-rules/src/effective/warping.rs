//! The full Warping chain (points, score, owed Virtues/Flaws, granted fills)
//! plus Decrepitude and aging-linked Characteristic drops — Decrepitude and
//! Warping share one mechanical family (both are advancement-curve-derived
//! scores accrued from points that gate off-budget V/F grants; aging drops
//! are the Characteristic-side consequence of the same accrual). Split out of
//! `might_warping.rs` (Viktor's V1 architecture finding, round 2 — that file
//! was a leftover bucket of ~10 unrelated domains after the round-1
//! `effective.rs` split); pure code motion, no behavior change.

use super::*;
use crate::house::WarpingExemption;

/// The Warping Points granted by [`Effect::WarpingGrant`] (Warped by Magic → 5)
/// and [`Effect::WarpingGrantParam`] (Raised from the Dead, D69/X7b-e: `base_points`
/// plus one per year named by the owning selection's own parameter — an
/// unanswered parameter contributes 0 extra years), summed across selections
/// and derived grants. Neither variant carries a *score* field (D77.3 dropped
/// `WarpingGrant`'s) — the Warping Score is derived by inverting the
/// advancement curve over the point total (see [`warping_score`]), so the
/// score is computed from points alone and a stored score could only ever
/// disagree with it.
fn warping_grant_points_in(selections: &[Selection], ruleset: &Ruleset) -> u32 {
    let mut points = 0u32;
    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            match effect {
                Effect::WarpingGrant { points: p } => {
                    points += u32::from(*p);
                }
                Effect::WarpingGrantParam { param, base_points } => {
                    let years = selection
                        .params
                        .get(param)
                        .and_then(SelectionParamValue::as_single)
                        .and_then(|v| v.as_str().parse::<u32>().ok())
                        .unwrap_or(0);
                    points += u32::from(*base_points) + years;
                }
                // Every other Effect variant grants no Warping Points. Listed
                // explicitly (not a wildcard `_`) so a new variant is a compile
                // error here, via `irrelevant_effect_variants_except!` — the
                // plain `irrelevant_effect_variants!()` macro is not reusable
                // here: it lists `WarpingGrant`/`WarpingGrantParam` themselves as
                // "irrelevant" (correctly, for ITS callers, which do not care
                // about Warping), which would collide with the two unconditional
                // arms above and never reach them.
                irrelevant_effect_variants_except!(WarpingGrant, WarpingGrantParam) => {}
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
/// Source: ArMDE:16464-16475.
pub fn warping_points_total(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .warping_points
        .saturating_add(warping_grant_points_in(
            selections_for_effects(entity, ruleset).as_ref(),
            ruleset,
        ))
        .saturating_add(house_conditional_warping_points(entity, ruleset))
}

/// The Warping Points that DETERMINE how many V/F are owed from Warping: the
/// stored points plus grant points from bought selections and non-warping grants
/// ([`entity_grants_base`]) ONLY. The owed warping fills are deliberately excluded
/// so a fill can never raise the score that decides how many fills are owed — the
/// recursion guard against the self-amplifying `warped_by_magic` feedback loop.
/// Source: ArMDE:16553-16561.
fn warping_points_for_owed(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut base = entity.selections.clone();
    base.extend(entity_grants_base(entity, ruleset));
    entity
        .warping_points
        .saturating_add(warping_grant_points_in(&base, ruleset))
        .saturating_add(house_conditional_warping_points(entity, ruleset))
}

/// Whether `entity` holds, among its effective (bought ∪ House/mythic/VF-
/// granted, non-warping-fill) selections, any Virtue or Flaw ArMDE:2280 reads
/// as "faerie-related" — the gate on Merinita's conditional Warping Point
/// ([`merinita_warping_points`]).
///
/// D81.14 (`docs/vf-audit/decisions.md`): an item counts if EITHER
/// `PointItem::faerie_related` is set (Faerie Friend, Faerie Upbringing,
/// Susceptibility to Faerie Power — three entries outside the realm system
/// entirely), OR it carries a realm association that resolves to
/// [`Realm::Faerie`] via the existing [`item_has_realm_association`] +
/// [`resolve_realm`] chain (Faerie Blood, Strong Faerie Blood, and Bound to /
/// Realm Stigmatic / Necessary Aura / Folk Magic when their own `realm`
/// parameter names Faerie). `virtue.faerie_magic` itself — the House's own
/// grant every Merinita magus holds by construction — carries neither, so it
/// correctly never counts.
///
/// Uses the same selection base as [`warping_points_for_owed`] (bought ∪
/// non-warping-fill grants): the warping-owed fills themselves are excluded
/// so a player cannot fill an owed-warping Flaw slot with a faerie-related
/// pick to retroactively dodge this very point.
fn has_faerie_related_vf(entity: &Entity, ruleset: &Ruleset) -> bool {
    let mut base = entity.selections.clone();
    base.extend(entity_grants_base(entity, ruleset));
    base.iter().any(|selection| {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            return false;
        };
        item.faerie_related
            || (item_has_realm_association(item, selection)
                && resolve_realm(item, selection, entity.concept_realm).realm == Realm::Faerie)
    })
}

/// The conditional Warping Point `entity`'s own House's Mystery inflicts at
/// creation, per the House's data-driven
/// [`crate::house::House::conditional_warping`] field — e.g. Merinita's "Any
/// magus in this House without a faerie-related Virtue or Flaw has a Warping
/// Point, inflicted to allow initiation into the Mystery" (ArMDE:2280). 0 for
/// an entity with no House, a House absent the field entirely, or one whose
/// stated exemption holds; the House's own `points` otherwise.
///
/// Generic over House by construction: there is no House-id branch here at
/// all, so a second House's analogous Mystery-initiation clause (Houses of
/// Hermes — Mystery Cults/True Lineages/Societas each define several) is a
/// `rules/core/houses.json` change, never a new Rust function.
fn house_conditional_warping_points(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let Some(house_id) = &entity.house else {
        return 0;
    };
    let Some(house) = ruleset.house(house_id) else {
        return 0;
    };
    let Some(conditional) = &house.conditional_warping else {
        return 0;
    };
    if warping_exemption_holds(conditional.unless, entity, ruleset) {
        0
    } else {
        conditional.points
    }
}

/// Whether `entity` satisfies the named [`WarpingExemption`] from a House's
/// conditional Warping clause. An exhaustive match so a new variant is a
/// compile error here until handled.
fn warping_exemption_holds(
    exemption: WarpingExemption,
    entity: &Entity,
    ruleset: &Ruleset,
) -> bool {
    match exemption {
        WarpingExemption::FaerieRelatedVf => has_faerie_related_vf(entity, ruleset),
    }
}

/// The Warping Score used to decide the owed warping V/F: [`warping_points_for_owed`]
/// inverted through the advancement curve (owed fills excluded — the recursion
/// guard). Source: ArMDE:16553-16561.
fn warping_score_for_owed(entity: &Entity, ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement
        .score_for_xp(warping_points_for_owed(entity, ruleset))
}

/// The character's derived Warping Score: [`warping_points_total`] inverted through
/// the (Ability) advancement curve (Warping rises "like an Ability": cumulative
/// 5/15/30/50/75, so 15 points → Warping Score 2). Source: ArMDE:16464-16475.
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
/// stored on the entity as a resolved value. Source: ArMDE:7019-7021, :16464-16475.
pub fn warping(entity: &Entity, ruleset: &Ruleset) -> Warping {
    Warping {
        score: warping_score(entity, ruleset),
        points: warping_points_total(entity, ruleset),
    }
}

/// The category slug a warping-owed supernatural Minor Virtue must belong to
/// (ArMDE:16559, "a supernatural Minor Virtue"). The category taxonomy is
/// data; this names the slug the rule's "supernatural" wording maps to.
const WARPING_SUPERNATURAL_CATEGORY: &str = "supernatural";

/// Stable `choice_key` prefix for each owed Minor Flaw slot (`…0`, `…1`).
pub(crate) const WARPING_MINOR_FLAW_KEY: &str = "warping.minor_flaw.";
/// Stable `choice_key` prefix for the owed supernatural Minor Virtue slot.
pub(crate) const WARPING_SUPERNATURAL_VIRTUE_KEY: &str = "warping.supernatural_virtue.";
/// Stable `choice_key` prefix for each owed Major Flaw slot.
pub(crate) const WARPING_MAJOR_FLAW_KEY: &str = "warping.major_flaw.";

/// The Virtues and Flaws a character owes from its Warping Score, per "Effects of
/// Warping" (ArMDE:16547-16561). These are auto-granted, off-budget V/F
/// (never counted against the creation Virtue/Flaw budget), filled by the player
/// choosing specific items (stored in [`Entity::warping_choices`]). Derived, never
/// stored as a resolved value.
///
/// Hermetic magi are exempt: Warping makes them prone to Wizard's Twilight
/// instead ("This replaces the normal effects", :16551), which this slice does
/// NOT model — a magus always owes zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WarpingOwed {
    /// Owed Minor Flaws: 1 at Warping Score 1 (ArMDE:16553), 2 at Score 3 (ArMDE:16557).
    pub minor_flaws: u8,
    /// Owed supernatural Minor Virtues: 1 at Warping Score 5 (ArMDE:16559), else 0.
    pub minor_supernatural_virtues: u8,
    /// Owed Major Flaws: 1 at Warping Score 6 and every point thereafter (ArMDE:16561).
    pub major_flaws: u8,
}

impl WarpingOwed {
    /// The owed V/F for a non-magus at Warping Score `score` — the pure threshold
    /// curve of "Effects of Warping". Source: ArMDE:16553-16561.
    pub fn from_score(score: u8) -> Self {
        WarpingOwed {
            // A Minor Flaw at Warping Score 1 (ArMDE:16553); a second at Score 3 (ArMDE:16557).
            minor_flaws: if score >= 3 {
                2
            } else if score >= 1 {
                1
            } else {
                0
            },
            // A supernatural Minor Virtue at Warping Score 5 (ArMDE:16559).
            minor_supernatural_virtues: u8::from(score >= 5),
            // A Major Flaw at Warping Score 6, and every point thereafter (ArMDE:16561).
            major_flaws: score.saturating_sub(5),
        }
    }
}

/// The Virtues/Flaws `entity` owes from Warping. The untrained owe per the score
/// (derived via the recursion-guarded [`warping_score_for_owed`], so owed fills
/// never inflate the count); the Hermetically trained (`is_hermetically_trained`,
/// D56/A0's union of the profile flag with any selection carrying
/// [`Effect::ConfersHermeticTraining`]) are exempt and owe zero — Warping gives
/// them Wizard's Twilight instead (ArMDE:16551). An Abandoned Apprentice casting
/// spells is equally exposed to Twilight, so he must get the same exemption a
/// magus gets.
pub fn warping_owed(entity: &Entity, ruleset: &Ruleset) -> WarpingOwed {
    let profile = ruleset.profile(&entity.type_id);
    if is_hermetically_trained(entity, ruleset, profile) {
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
        item.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::WarpingGrant { .. } | Effect::WarpingGrantParam { .. }
            )
        })
    })
}

/// The owed warping V/F expressed as OPEN [`Grant`]s — one grant per owed slot,
/// each with a stable `choice_key` and the [`GrantConstraint`] its fill must
/// satisfy (a Minor Flaw, a supernatural Minor Virtue, or a Major Flaw). The list
/// length tracks the recursion-guarded Warping Score via [`warping_owed`]. The
/// frontend renders one picker per grant; validation resolves each pick against
/// its constraint. Source: ArMDE:16553-16561.
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
/// purposes. Budget-exempt, exactly like House grants — but **not** cap-exempt:
/// these folded copies count toward `max_per_target`, `max_total` and
/// `max_share_of_kind`, because [`crate::validation::validate`] runs those checks
/// against the folded bought-plus-granted list. A pick carrying
/// [`Effect::WarpingGrant`] is dropped (ineligible — the recursion guard), so a
/// warping fill can never feed Warping Points back into the owed count.
/// Source: ArMDE:16553-16561.
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
/// Source: ArMDE:16617.
pub fn decrepitude_points_total(entity: &Entity) -> u32 {
    entity.aging_points.values().map(|p| u32::from(*p)).sum()
}

/// The character's derived Decrepitude Score: [`decrepitude_points_total`] inverted
/// through the (Ability) advancement curve (Decrepitude rises "like an Ability",
/// 5×new score, so 17 aging points → Decrepitude 2). Source: ArMDE:16617.
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
/// Source: ArMDE:16579, :16613.
///
/// Crate-internal primitive: the frontend consumes the surfaced
/// [`characteristic_aging_drops`] map (which wraps this per-Characteristic), so
/// this single-Characteristic query is not part of the curated public API.
///
/// # Unaging drops nothing
///
/// Returns 0 for a character carrying an [`AgingEffect::NoAging`] item: "In game
/// terms, your aging points do not decrease your Characteristics, only building up
/// to give you Decrepitude points" (`ArMDE:5189`; Bound to (Role) "also includes the
/// effects of the Unaging Virtue" at `ArMDE:5743`). The second half of that sentence is
/// why [`decrepitude_points_total`] and [`decrepitude_score`] are deliberately
/// **not** gated the same way — the points still accrue and still build
/// Decrepitude, at everybody else's rate. The *appearance* is a separate exemption
/// ([`AgingEffect::NoApparentAging`]), applied in `aging.rs`.
///
/// Source: ArMDE:5189, :5743.
pub(crate) fn aging_drops(
    entity: &Entity,
    ruleset: &Ruleset,
    characteristic: Characteristic,
) -> u32 {
    if suppresses_characteristic_aging(entity, ruleset) {
        return 0;
    }
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

/// Whether the character's Characteristics are exempt from aging drops — i.e.
/// whether he carries an [`AgingEffect::NoAging`] item.
///
/// It gates the Characteristic drop **only**. Unaging carries this tag alongside
/// `no_apparent_aging`, Bound to (Role) carries it alone (`ArMDE:5743` advances the
/// apparent age "in line with their physical age"), and a Bee King carries neither
/// — "do not appear to age" (`ArMDE:3488`) is about the appearance and nothing else.
///
/// Source: ArMDE:5189, :5743, :3488.
fn suppresses_characteristic_aging(entity: &Entity, ruleset: &Ruleset) -> bool {
    selections_for_effects(entity, ruleset)
        .iter()
        .filter_map(|selection| ruleset.point_items.get(&selection.item_ref))
        .flat_map(|item| &item.effects)
        .any(|effect| {
            matches!(
                effect,
                Effect::AgingMod {
                    kind: AgingEffect::NoAging,
                    ..
                }
            )
        })
}

/// The effective value of `characteristic` after aging: the bought score lowered
/// by the DERIVED aging drops ([`aging_drops`]), with any free
/// [`Effect::CharacteristicScoreDelta`] bonus (Giant
/// Blood +1 Str/Sta, Dwarf -1) then added on top — so an aged Giant-Blood score
/// can still reach ±6. The aging drop lowers the *bought* score (its threshold is
/// the bought score); the free delta is a separate additive layer. This is what
/// DERIVED / play stats consume; it is deliberately **not** what creation-legality
/// reads (the point-buy budget check in `validation.rs` reads the un-aged bought
/// score from `entity.characteristics`), so entering an already-aged character
/// cannot retroactively make its point-buy illegal.
///
/// # There is no floor, deliberately
///
/// `ArMDE:16579` gives the drop condition and names no minimum, so neither does
/// this function. It carried an engine-invented one (-5, then -10) until
/// 2026-09-15; a clamp at either value says something the book does not — that a
/// character decrepit with age can be no weaker than a freshly-built grog. The
/// clamp existed to keep the derived score bounded, but the score is bounded by
/// the data already: [`Entity::aging_points`] is a `u8` per Characteristic and
/// each drop costs one point more than the last, so the drops terminate on their
/// own. Nothing invented is needed to make the arithmetic safe.
///
/// Source: ArMDE:16579.
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
    let drops = i32::try_from(aging_drops(entity, ruleset, characteristic)).unwrap_or(i32::MAX);
    bought.saturating_sub(drops) + characteristic_score_bonus(entity, ruleset, characteristic)
}
