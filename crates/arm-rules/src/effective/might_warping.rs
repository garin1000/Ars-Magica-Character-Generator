//! Gift/Supernatural, Confidence, item/power budgets, Might, True Faith, the
//! full Warping chain (points/score/owed/grants), Decrepitude/aging drops,
//! Reputation grants, Supernatural free slots, and age-based Ability caps.
//! Split out of `effective.rs` (Viktor's V4 architecture finding — 93 free
//! functions across 7 unrelated domains in one file); pure code motion, no
//! behavior change. Originally the file's "Phase 7" section.

use super::*;

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
/// stored. Source: Ars Magica - Definitive Edition (Core Rules).md:2520-2526,
/// 4900-4902.
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
/// Ars Magica - Definitive Edition (Core Rules).md:4347-4349, :4842-4846.
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
/// side of the item-level budget bar. Summed across every device. Source: Ars
/// Magica - Definitive Edition (Core Rules).md:4347-4349.
pub fn item_level_used(entity: &Entity) -> u32 {
    entity.devices.iter().map(|d| u32::from(d.level)).sum()
}

/// The character's derived power-levels budget: base 0 plus every
/// [`Effect::PowerLevels`] grant (Demonic Blood 30, Demonic Powers +20, Strong
/// Angelic Heritage 30), summed. The being's `powers` are charged against it,
/// mirroring [`item_level_budget`]. Source: Ars Magica 5e - Realms of Power -
/// The Infernal.md:4122, :4142; Ars Magica 5e - Realms of Power - The Divine
/// (Revised).md:1977.
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
/// power-levels budget bar. Source: Ars Magica 5e - Realms of Power - The
/// Infernal.md:4122.
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
/// Infernal Might 5, Demonic Might +2 → effective 7. Source: Ars Magica 5e -
/// Realms of Power - Magic.md:1470-1472; Ars Magica 5e - Realms of Power -
/// The Infernal.md:4120, :4136.
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
/// `u8`. Derived, never stored. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:5169-5171.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16464-16475.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16553-16561.
fn warping_points_for_owed(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut base = entity.selections.clone();
    base.extend(entity_grants_base(entity, ruleset));
    entity
        .warping_points
        .saturating_add(warping_grant_points_in(&base, ruleset))
}

/// The Warping Score used to decide the owed warping V/F: [`warping_points_for_owed`]
/// inverted through the advancement curve (owed fills excluded — the recursion
/// guard). Source: Ars Magica - Definitive Edition (Core Rules).md:16553-16561.
fn warping_score_for_owed(entity: &Entity, ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement
        .score_for_xp(warping_points_for_owed(entity, ruleset))
}

/// The character's derived Warping Score: [`warping_points_total`] inverted through
/// the (Ability) advancement curve (Warping rises "like an Ability": cumulative
/// 5/15/30/50/75, so 15 points → Warping Score 2). Source: Ars Magica - Definitive Edition (Core Rules).md:16464-16475.
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
/// stored on the entity as a resolved value. Source: Ars Magica - Definitive
/// Edition (Core Rules).md:7019-7021, :16464-16475.
pub fn warping(entity: &Entity, ruleset: &Ruleset) -> Warping {
    Warping {
        score: warping_score(entity, ruleset),
        points: warping_points_total(entity, ruleset),
    }
}

/// The category slug a warping-owed supernatural Minor Virtue must belong to
/// (Ars Magica - Definitive Edition (Core Rules).md:16559, "a supernatural Minor Virtue"). The category taxonomy is
/// data; this names the slug the rule's "supernatural" wording maps to.
const WARPING_SUPERNATURAL_CATEGORY: &str = "supernatural";

/// Stable `choice_key` prefix for each owed Minor Flaw slot (`…0`, `…1`).
pub(crate) const WARPING_MINOR_FLAW_KEY: &str = "warping.minor_flaw.";
/// Stable `choice_key` prefix for the owed supernatural Minor Virtue slot.
pub(crate) const WARPING_SUPERNATURAL_VIRTUE_KEY: &str = "warping.supernatural_virtue.";
/// Stable `choice_key` prefix for each owed Major Flaw slot.
pub(crate) const WARPING_MAJOR_FLAW_KEY: &str = "warping.major_flaw.";

/// The Virtues and Flaws a character owes from its Warping Score, per "Effects of
/// Warping" (Ars Magica - Definitive Edition (Core Rules).md:16547-16561). These are auto-granted, off-budget V/F
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
    /// curve of "Effects of Warping". Source: Ars Magica - Definitive Edition (Core Rules).md:16553-16561.
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
/// owe zero — Warping gives them Wizard's Twilight instead (Ars Magica - Definitive Edition (Core Rules).md:16551).
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
/// its constraint. Source: Ars Magica - Definitive Edition (Core Rules).md:16553-16561.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16553-16561.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16617.
pub fn decrepitude_points_total(entity: &Entity) -> u32 {
    entity.aging_points.values().map(|p| u32::from(*p)).sum()
}

/// The character's derived Decrepitude Score: [`decrepitude_points_total`] inverted
/// through the (Ability) advancement curve (Decrepitude rises "like an Ability",
/// 5×new score, so 17 aging points → Decrepitude 2). Source: Ars Magica - Definitive Edition (Core Rules).md:16617.
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
///
/// # Unaging drops nothing
///
/// Returns 0 for a character carrying an [`AgingEffect::NoAging`] item: "In game
/// terms, your aging points do not decrease your Characteristics, only building up
/// to give you Decrepitude points" (`:5189`; Bound to (Role) "also includes the
/// effects of the Unaging Virtue" at `:5743`). The second half of that sentence is
/// why [`decrepitude_points_total`] and [`decrepitude_score`] are deliberately
/// **not** gated the same way — the points still accrue and still build
/// Decrepitude, at everybody else's rate. The *appearance* is a separate exemption
/// ([`AgingEffect::NoApparentAging`]), applied in `aging.rs`.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:5189, :5743.
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
/// `no_apparent_aging`, Bound to (Role) carries it alone (`:5743` advances the
/// apparent age "in line with their physical age"), and a Bee King carries neither
/// — "do not appear to age" (`:3488`) is about the appearance and nothing else.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:5189, :5743, :3488.
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
    let drops = i32::try_from(aging_drops(entity, ruleset, characteristic)).unwrap_or(i32::MAX);
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
/// authorizing starting Reputations. Source: Ars Magica - Definitive Edition
/// (Core Rules).md:2512-2514.
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2874.
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
/// age band table (Ars Magica - Definitive Edition (Core Rules).md:2366-2374). Data, not hardcoded: the bands live
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

/// The age cap for ONE ability, after any Virtue/Flaw that narrows it for
/// locality-dependent Abilities.
///
/// > The maximum scores at character creation for locality-dependent Abilities like
/// > Language, Area Lore, or Organization Lore, as well as some social Abilities,
/// > are half (round up) that which his age normally allows.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:6160 (Foreign
/// Upbringing). Which Abilities count as locality-dependent is catalogue data
/// (`locality_dependent`), because the passage's "as well as some social Abilities"
/// is deliberately open — the engine enforces the flag it is given rather than
/// guessing which social Abilities a saga counts.
///
/// The fraction rounds **up**, per the passage. Several such flaws would compose by
/// applying the smallest resulting cap, though no shipped Flaw pairs with another.
pub fn ability_age_cap(entity: &Entity, ruleset: &Ruleset, ability: &Id) -> Option<u8> {
    let base = age_ability_cap(entity, ruleset)?;
    if !ruleset
        .ability(ability)
        .is_some_and(|def| def.locality_dependent)
    {
        return Some(base);
    }
    let mut cap = base;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::LocalityAbilityCapFraction { num, den } = effect
                && *den > 0
            {
                // Ceiling division: "half (round up)".
                let numerator = u32::from(base) * u32::from(*num) + u32::from(*den) - 1;
                let fractioned = u8::try_from(numerator / u32::from(*den)).unwrap_or(base);
                cap = cap.min(fractioned);
            }
        }
    }
    Some(cap)
}
