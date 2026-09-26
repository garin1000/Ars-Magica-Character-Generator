//! The IPC read-out DTO layer: [`EffectiveScores`] — the score effects a
//! character's virtues/flaws produce, computed fresh from an `Entity` +
//! `Ruleset` on every call — the private per-domain field structs it is
//! assembled from, their assemblers, and [`effective_scores_loaded`] itself.
//!
//! Split out of `ruleset_io.rs`, which keeps the ruleset loader, the save/
//! export path helpers, and the aging commands: those are a different concern
//! under what used to be the same file, even though both are called from the
//! same `commands.rs` shims.

use std::collections::BTreeMap;

use arm_rules::{
    AbilityBonus, AbilityFloor, ArtBonus, Characteristic, CharacteristicBonus, Confidence, Entity,
    EntityTypeProfile, Grant, Id, LifeStageBudget, MagusMinimumAbility, MightScore, PointCeilings,
    ReputationType, RestrictedXpPool, Ruleset, Selection, SpellLevelCap, SupernaturalFreeSlots,
    ability_bonuses, ability_score_floors, aging_schedule, aging_total, art_bonuses,
    characteristic_aging_drops, characteristic_bonuses, characteristic_caps, characteristic_floors,
    characteristic_points_granted, checked_xp_allocation, compute_balance, confidence,
    decrepitude_score, effective_characteristics, effective_might, effective_point_ceilings,
    entity_grants, focus_points_budget, focus_points_used, is_hermetically_trained,
    item_level_budget, item_level_used, life_stage_spell_levels, longevity_bonus,
    magus_minimum_abilities, power_levels_budget, powers_used, reputation_grants, size,
    spell_level_caps, spell_levels_base, spell_levels_bonus, spell_levels_budget,
    spell_levels_used, spell_mastery_advancement_affinity, spell_mastery_floor, spell_mastery_xp,
    supernatural_free_slots, true_faith, warping, warping_owed_grants,
};
use serde::Serialize;

/// The score effects a character's virtues/flaws produce, for the frontend.
/// Ability bonuses are per-instance and only the non-zero ones are present (a
/// Puissant on "Brandenburg Lore" attaches to that row alone). Characteristic
/// caps/floors are the per-characteristic buy limits — keyed by the snake_case
/// characteristic name and present for all eight — that Great/Poor
/// (Characteristic) widen, so the UI clamps the spinners to them.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveScores {
    /// One entry per boosted ability instance (e.g. Puissant Ability +2).
    pub ability_bonuses: Vec<AbilityBonus>,
    /// One entry per boosted Art (e.g. Puissant Art +3).
    pub art_bonuses: Vec<ArtBonus>,
    /// Characteristic → highest buyable score (Great Characteristic raises it).
    pub characteristic_caps: BTreeMap<Characteristic, i32>,
    /// Characteristic → lowest buyable score (Poor Characteristic lowers it).
    pub characteristic_floors: BTreeMap<Characteristic, i32>,
    /// The point-buy cost of the entity's Characteristics, netting gains against
    /// spends — engine-authoritative, so the frontend never recomputes it.
    ///
    /// It exists because `ui/src/lib/derive.ts` used to hold its own copy of the
    /// point-buy table (audit findings VA1/GF1/GD4, raised independently by three
    /// reviewers): a second implementation of a rule the engine already owns, free
    /// to drift from `CharacteristicRules::total_cost` with nothing to catch it.
    pub characteristic_points_used: i32,
    /// Total experience the entity's Ability+Art spends demand, after Affinity
    /// reductions — the authoritative "spent" the UI shows (it must not recompute
    /// it without Affinity).
    pub xp_total_demand: u32,
    /// Experience drawn from the general pool by the allocation; restricted pools
    /// cover the rest.
    pub xp_general_used: u32,
    /// The general pool itself: the typed `Entity::xp_pool` for a directly-entered
    /// character, and for one built through its life stages the block that may fund
    /// anything — apprenticeship plus the years past the Gauntlet for a magus, later
    /// life for anyone else — plus the
    /// Skilled/Weak Parens adjustment. Engine-authoritative, because the base and the
    /// pool differ (240 against 300 with Skilled Parens) and no stored field holds
    /// the latter.
    pub xp_general_pool: u32,
    /// The signed Virtue/Flaw contribution folded into `xp_general_pool` (Skilled
    /// Parens +60, Weak Parens -60). Surfaced on its own because the bar's editable
    /// total is the base the player typed, not the pool: without this figure the two
    /// differ with nothing on screen to explain it — the same split the spell-levels
    /// bar makes with `spell_levels_bonus`.
    pub xp_general_bonus: i64,
    /// The most demand the pools can actually fund. Equals `xp_total_demand` iff the
    /// spend is legal; `xp_total_demand - xp_max_flow` is the overspend.
    ///
    /// The bar needs this to show an overspent pool: `xp_general_used` is a max-flow
    /// value capped by the pool itself, so `pool - general_used` can never go
    /// negative no matter how far the spend exceeds the pool.
    pub xp_max_flow: u32,
    /// The restricted experience pools (Educated/Warrior/Privileged, and the
    /// life-stage blocks) with how much of each the allocation consumes, for the
    /// per-pool XP bar. Each carries its `origin`, so the bar labels it rather than
    /// inferring a name from its ability list.
    pub restricted_xp_pools: Vec<RestrictedXpPool>,
    /// The experience the character's life stages earn, block by block, or `None`
    /// for a directly-entered character (where `xp_pool` is the authority). The
    /// guided flow shows this instead of an editable pool.
    pub life_stage: Option<LifeStageBudget>,
    /// The die-independent half of the character's aging: which rolls are owed,
    /// which are recorded, and every term of the AGING TOTAL except the die.
    /// `None` when the ruleset ships no aging rules at all — the same shape as
    /// [`Self::life_stage`], and for the same reason: an absent block stands the
    /// subsystem down rather than letting the payload invent a threshold.
    pub aging: Option<AgingReadout>,
    /// Net Characteristic-buy points granted by Improved / Weak Characteristics,
    /// on top of the ruleset's base `start_points`. Signed (Weak subtracts).
    pub characteristic_points_granted: i32,
    /// Free starting-score floors a virtue grants to an ability (e.g. Second
    /// Sight → Second Sight 1), for the ability row's effective-score display.
    pub ability_score_floors: Vec<AbilityFloor>,
    /// The character's derived Size (base 0; Large +1, Giant Blood +2, Small
    /// Frame −1, Dwarf −2), for the character-sheet Size readout.
    pub size: i32,
    /// Free effective-score bonuses to Characteristics (Giant Blood +1 Str/Sta,
    /// Dwarf −1), one per affected Characteristic, for the sheet to show the
    /// effective score alongside the bought one.
    pub characteristic_bonuses: Vec<CharacteristicBonus>,
    /// Effective Characteristic score after aging drops AND free virtue deltas,
    /// keyed by Characteristic — only the entries that differ from the bought
    /// score (the UI falls back to the bought score for the rest). The engine
    /// owns the floor clamp, so the UI never re-implements it.
    pub characteristic_effective: BTreeMap<Characteristic, i32>,
    /// Aging-drop count per Characteristic (only the non-zero entries), for the
    /// effective-score tooltip breakdown.
    pub characteristic_aging_drops: BTreeMap<Characteristic, u32>,
    /// Virtue/Flaw Selections the entity's House grants (derived, never persisted),
    /// so the V/F view renders them read-only without re-deriving. Emitted in the
    /// House's declared grant order for a stable UI + snapshot ordering.
    pub granted_selections: Vec<Selection>,
    /// The character type's virtue/flaw point ceilings, so the balance bar shows
    /// the budget without recomputing it in TS. Budget numbers stay
    /// engine-authoritative.
    pub virtue_budget: u32,
    pub flaw_budget: u32,
    /// The virtue/flaw points actually spent — `validation::compute_balance`'s own
    /// figure, the same one `export.rs`'s Markdown export and the
    /// over-budget/unbalanced-Virtues validation issues already read.
    ///
    /// Surfaced so the balance bar stops re-deriving this a third time in
    /// TypeScript (audit finding G1, round 4): a second, independent
    /// implementation of a rule the engine already owns, free to drift from
    /// `compute_balance` with nothing to catch it — the same defect class as
    /// [`Self::characteristic_points_used`] above (VA1/GF1/GD4).
    pub virtue_points: i32,
    pub flaw_points: i32,
    /// The magus's effective spell-levels budget (base + Skilled/Weak Parens
    /// modifiers + the levels its post-Gauntlet years bought) — the "available"
    /// side of the spell-levels bar. The base is the per-character
    /// `spell_levels_override` when set, else the type profile's base.
    pub spell_levels_budget: u32,
    /// The type profile's base spell-levels budget (120 for a magus), surfaced so
    /// the override field's placeholder shows the data-driven default rather than a
    /// hardcoded literal. Ignores the per-character override and V/F modifiers.
    pub spell_levels_profile_base: u32,
    /// The V/F contribution to the budget on its own (Skilled Parens +30, Weak
    /// Parens −30; signed, 0 when none).
    ///
    /// The budget's three parts — [`Self::spell_levels_profile_base`], this, and
    /// [`Self::spell_levels_life_stage`] — are surfaced separately because they come
    /// from three different rules and the bar labels each, the way the XP bar lists
    /// its extra pools beside the general pool; folding them into
    /// [`Self::spell_levels_budget`] alone would leave the player an unexplained
    /// total. The identity is
    /// `base + bonus + life_stage == spell_levels_budget`.
    pub spell_levels_bonus: i64,
    /// The levels of spells the magus's years past its Gauntlet bought: the player's
    /// chosen slice of the fungible 30-points-a-year, where "Each point can be an
    /// experience point in an Art or Ability or one level of spell"
    /// (ArMDE:2471). 0 for a magus standing
    /// at its Gauntlet and for anyone who serves no apprenticeship.
    ///
    /// **Not a second budget** like apprenticeship's 120 levels (`ArMDE:2435`), which are
    /// the type profile's `spell_levels` and reach the UI as
    /// [`Self::spell_levels_profile_base`] — these are added on top of it.
    pub spell_levels_life_stage: u32,
    /// The spell levels the chosen spells consume — the "used" side of the bar.
    pub spell_levels_used: u32,
    /// Per-Technique/Form maximum learnable spell level (Te + Fo + Int + Magic
    /// Theory + 3), so the spell picker greys a spell above the cap without
    /// recomputing the derivation in JS. Empty for an entity that is not
    /// Hermetically trained (no Spells tab) — by profile (a real magus) or by
    /// selection (D56's Abandoned Apprentice). Engine-authoritative; the UI
    /// only reads it.
    pub spell_level_caps: Vec<SpellLevelCap>,
    /// The Hermetic minimum-Ability checklist: what the Order demands (ArMDE:2437) and
    /// what the rulebook recommends (ArMDE:2451-2461), each with the character's bought
    /// score and whether it suffices. Empty for a non-magus, exactly like
    /// [`Self::spell_level_caps`] — `ArMDE:2437` is about admission to the Order.
    /// Engine-authoritative: the same reading the `magus_minimum_ability` /
    /// `magus_recommended_ability` findings come from.
    pub magus_minimum_abilities: Vec<MagusMinimumAbility>,
    /// Effective Confidence Score / Points (type default + V/F), for the read-only
    /// Confidence readout. 0/0 for grogs (who have no Confidence).
    pub confidence_score: u8,
    pub confidence_points: u8,
    /// The Gift's free Supernatural-Ability slots: how many the character has
    /// (1 for a Gifted non-magus, else 0) and how many are already used. The
    /// ability picker greys a Supernatural Ability when `used >= total` and it is
    /// not already granted by a Virtue.
    pub supernatural_free_total: u8,
    pub supernatural_free_used: u8,
    /// The Reputation grants the character's V/F confer, so the UI only offers a
    /// Reputation add-control (pre-filled kind/score) when one exists.
    pub reputation_grants: Vec<ReputationGrant>,
    /// Derived Warping Score / Points: the UNIFIED total of stored Warping Points
    /// (`Entity::warping_points`) plus any granted by V/F (Warped by Magic → +5),
    /// with the score derived by inverting the advancement curve (15 points → 2).
    /// 0/0 when there is no Warping. Engine-authoritative; never recomputed in JS.
    pub warping_score: u8,
    pub warping_points: u32,
    /// One OPEN grant per owed warping slot (stable `choice_key` + the constraint
    /// its fill must satisfy), so the UI renders one picker per slot filtered to
    /// eligible items. Empty for magi and characters owing nothing.
    pub warping_owed_grants: Vec<Grant>,
    /// Derived Decrepitude Score: the sum of accrued aging points across all
    /// Characteristics (`Entity::aging_points`) inverted through the advancement
    /// curve (17 aging points → Decrepitude 2). 0 when there are no aging points.
    pub decrepitude_score: u8,
    /// Derived True Faith Score granted by V/F (True Faith → 1); 0 when none.
    pub true_faith_score: u8,
    /// Derived starting enchanted-device level budget (Magic Items +25, Redcap
    /// 50); 0 when none. The character starts with this many levels of devices.
    pub item_level_budget: u32,
    /// The total device level the entity's `devices` consume — the "used" side of
    /// the item-level budget bar (engine-authoritative; the UI never recomputes it).
    pub item_level_used: u32,
    /// Derived Spell-Mastery XP pool (Mastered Spells +50 each); 0 when none.
    pub spell_mastery_xp: u32,
    /// Mastery-score floor every known spell gets (Flawless Magic → 1); 0 = none.
    pub spell_mastery_floor: u8,
    /// The Affinity applying to every Spell-Mastery Advancement Total, as the
    /// authored `[num, den]` pair ("counts as num/den of itself"; Flawless Magic
    /// → `[2, 1]`, halving the XP each mastery point costs); `None` when no
    /// grant reduces the cost. The *ratio* crosses, not a "doubled" flag,
    /// because it is rules data: the engine prices any pair through
    /// `effective/xp.rs::charged_cost`, so the UI must be able to charge the
    /// same reduced cost for any pair the catalogue authors — a new Virtue with
    /// a different Affinity has to stay the data-only change the "catalogue size
    /// is data, never code" invariant promises.
    pub spell_mastery_advancement_affinity: Option<[u8; 2]>,
    /// The supernatural being's effective Might Score + Realm (base + same-Realm
    /// Virtue grants), or `None` for an ordinary character. Engine-authoritative.
    pub might: Option<MightScore>,
    /// Derived power-levels budget the being's Might Virtues grant (Demonic Blood
    /// 30, Demonic Powers +20); 0 when none — the "available" side of the bar.
    pub power_levels_budget: u32,
    /// The total power level the being's `powers` consume — the "used" side of the
    /// power-levels bar (engine-authoritative; the UI never recomputes it).
    pub power_levels_used: u32,
    /// The Focus Power point pool the character's copies of the Virtue grant (25
    /// each, `ArMDE:3899`, `ArMDE:3903`); 0 when none. A *second* power currency,
    /// carried separately so neither bar can subsidise the other.
    pub focus_points_budget: u32,
    /// The points the character's `focus_powers` spend — 2 per level of effect
    /// plus 1 per point of Penetration (`ArMDE:3899`).
    pub focus_points_used: u32,
}

/// Everything about a character's aging that does **not** depend on a die.
///
/// The stress die is player input the entity must never store (`ArMDE:16567` — the
/// engine has no `rand` dependency and never will), so the roll itself is a
/// command the player triggers. Every field here, by contrast, is a pure function
/// of `(entity, ruleset)`, which is exactly why it rides on the always-recomputed
/// effective-scores payload rather than on a separate request: it can never be
/// stale, and the UI never has to ask for it.
///
/// Source: ArMDE:16563-16617.
#[derive(Debug, Clone, Serialize)]
pub struct AgingReadout {
    /// The first age at which a roll is owed — 36 under the shipped rules, the
    /// Winter after the character turns 35 (`ArMDE:16565`).
    pub first_roll_age: u32,
    /// The age aging begins AFTER (`ArMDE:16565`) — the ruleset's own `start_age`,
    /// surfaced for the explanatory line so the UI never prints the threshold as
    /// a literal.
    pub begins_after_age: u32,
    /// Every year the character owes a roll for, ascending, each marked with
    /// whether the log already records it.
    pub schedule: Vec<AgingScheduleYear>,
    /// `schedule.len()`, so the UI never counts a rules-defined set itself.
    pub rolls_owed: u32,
    /// How many of those owed years the log already records.
    pub rolls_recorded: u32,
    /// ⌈age / divisor⌉ at the character's **actual** age (`ArMDE:16577`); 0 when no
    /// age is entered, because there is then no age term to compute.
    pub age_modifier: i32,
    /// The Living Conditions modifier the total SUBTRACTS (`ArMDE:16569`, `ArMDE:16571`),
    /// the chosen rows and the Virtue/Flaw contributions together.
    pub living_conditions_modifier: i32,
    /// The Longevity Ritual modifier the total SUBTRACTS (`ArMDE:16569`); 0 with no
    /// ritual, or with one whose bonus the player has not entered.
    pub longevity_modifier: i32,
    /// Σ of the Virtue/Flaw aging-ROLL modifiers — ADDED with their stored sign
    /// (Faerie Blood's -1, `ArMDE:3801`). The book's three-line formula does not name
    /// this term, but [`AgingReadout::fixed_total`] has always included it, so the
    /// read-out cannot state its own arithmetic without it: a character with an
    /// aging-roll Flaw read "+4 (age) 0 (living conditions) 0 (Longevity Ritual) =
    /// stress die +3". Surfaced rather than derived in the UI, so the sentence's
    /// terms and its total come from the one engine computation.
    pub trait_modifier: i32,
    /// Whether the `ArMDE:16575` clamp stands over this character — he holds a
    /// Longevity Ritual and has not yet reached the clamp's age.
    ///
    /// **Named apart from [`arm_rules::AgingTotal::capped_by_longevity`] on
    /// purpose.** That one is a per-ROLL fact ("this total was cut down"); this
    /// is a standing predicate about the character, true even in a year whose
    /// total never reached the ceiling.
    pub longevity_clamp_active: bool,
    /// The whole non-die half of the AGING TOTAL, so the UI adds only the number
    /// the player typed: `age_modifier - living_conditions_modifier -
    /// longevity_modifier`, plus the Virtue/Flaw aging-roll modifiers (Faerie
    /// Blood's -1, `ArMDE:3801`), which the book's three-line formula does not name
    /// but which are just as die-independent.
    pub fixed_total: i32,
}

/// One year of [`AgingReadout::schedule`]: the age the roll is owed at, the
/// calendar year it falls in when the character has a birth year, and whether it
/// has been rolled.
#[derive(Debug, Clone, Serialize)]
pub struct AgingScheduleYear {
    /// The age the character reaches in this year.
    pub age: u32,
    /// `birth_year + age`, or `None` when no birth year is recorded.
    pub year: Option<i32>,
    /// Whether the aging log already carries a resolved entry for this age.
    pub recorded: bool,
}

/// Reads the die-independent half of a character's aging.
///
/// The terms come from one probe of the engine's own [`aging_total`] at a die of
/// zero rather than from a second derivation here: that keeps the read-out and
/// the roll the player will actually make arithmetically identical by
/// construction, sign conventions included.
///
/// Source: ArMDE:16563-16617.
fn aging_readout(entity: &Entity, ruleset: &Ruleset) -> Option<AgingReadout> {
    let rules = ruleset.aging()?;
    let schedule = aging_schedule(entity, ruleset);
    // No age entered means no age term (`ArMDE:16577` asks for the actual age, and
    // there is none) — and an empty schedule, so nothing is owed either.
    let age = entity.age.unwrap_or(0);
    let terms = aging_total(entity, ruleset, age, 0)?;

    Some(AgingReadout {
        first_roll_age: rules.first_roll_age(),
        begins_after_age: rules.start_age,
        rolls_owed: u32::try_from(schedule.len()).unwrap_or(u32::MAX),
        rolls_recorded: u32::try_from(schedule.iter().filter(|year| year.recorded).count())
            .unwrap_or(u32::MAX),
        schedule: schedule
            .into_iter()
            .map(|year| AgingScheduleYear {
                age: year.age,
                year: year.year,
                recorded: year.recorded,
            })
            .collect(),
        age_modifier: terms.age_modifier,
        living_conditions_modifier: terms.living_conditions.total,
        longevity_modifier: terms.longevity_bonus,
        trait_modifier: terms.trait_modifier,
        longevity_clamp_active: rules
            .longevity_clamp
            .as_ref()
            .is_some_and(|clamp| age < clamp.until_age)
            && longevity_bonus(entity, ruleset).is_some(),
        // The probe's die was 0, so its uncapped total IS the non-die half.
        fixed_total: terms.uncapped_total,
    })
}

/// A Reputation a Virtue/Flaw authorizes the character to start with. The UI
/// renders one row per grant — kind and score come from here, `content` is
/// player-supplied — so a grant is a slot waiting to be described, never an
/// "Add" button that can be pressed twice.
///
/// `kind` is `None` for a player-chosen-type grant (Famous), which the panel
/// renders as a type `<select>`. Renamed from the engine's `reputation_type`
/// only; `source` and `score` pass through untouched.
#[derive(Debug, Clone, Serialize)]
pub struct ReputationGrant {
    /// The Virtue/Flaw that opened this slot, so the row can say why it exists.
    pub source: Id,
    /// The Reputation type the grant fixes, or `None` when player-chosen.
    pub kind: Option<ReputationType>,
    pub score: u8,
    /// The upper bound of a stated range (D11/Q5), or `None` when the score is
    /// exact. Outsider alone states one ("a bad Reputation of level 1 to 3",
    /// ArMDE:6554); every other grant leaves this absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_score: Option<u8>,
}

/// The XP-bar slice of [`EffectiveScores`]: the max-flow allocation's
/// demand/used/pool/bonus/restricted-pools, plus the life-stage budget the
/// guided flow substitutes for an editable pool. Grouped because all of it
/// reads off one call to [`checked_xp_allocation`] plus one sibling
/// life-stage lookup.
struct XpFields {
    total_demand: u32,
    general_used: u32,
    general_pool: u32,
    general_bonus: i64,
    max_flow: u32,
    restricted: Vec<RestrictedXpPool>,
    life_stage: Option<LifeStageBudget>,
}

/// Audit finding K1 (round 2): this used to call the raw, now-`pub(crate)`
/// `xp_allocation` directly, with no check that the entity's selections fit
/// the flow-solve node bound. A crafted `.armc` with an oversized
/// `ability_scores`/`art_scores`/`spells` array reached the solver's internal
/// `assert!` on a plain File → Open (`effective_scores` is one of the calls
/// `revalidate()` fires alongside `validateEntity`, so the panic could win the
/// race against the friendly `CODE_XP_SOLVE_BOUND_EXCEEDED` rejection). An
/// over-bound entity now degrades to all-zero/empty XP fields instead: the
/// concurrent `validate_entity` call (which runs the identical check via
/// `checked_xp_allocation`) is what actually tells the user why, so this
/// command staying silent about the specific reason is not a regression —
/// what matters is that it returns a value instead of unwinding.
fn xp_fields(entity: &Entity, ruleset: &Ruleset) -> XpFields {
    let life_stage = ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset));
    match checked_xp_allocation(entity, ruleset) {
        Ok(allocation) => XpFields {
            total_demand: allocation.total_demand,
            general_used: allocation.general_used,
            general_pool: allocation.general_pool,
            general_bonus: allocation.general_bonus,
            max_flow: allocation.max_flow,
            restricted: allocation.restricted,
            life_stage,
        },
        Err(_) => XpFields {
            total_demand: 0,
            general_used: 0,
            general_pool: 0,
            general_bonus: 0,
            max_flow: 0,
            restricted: Vec::new(),
            life_stage,
        },
    }
}

/// The spell-levels-bar slice of [`EffectiveScores`]: budget, its three named
/// components, the used side, and the two magus-only checklists that ride on
/// the same "does this character even have Spells" gate.
struct SpellFields {
    budget: u32,
    profile_base: u32,
    bonus: i64,
    life_stage: u32,
    used: u32,
    level_caps: Vec<SpellLevelCap>,
    minimum_abilities: Vec<MagusMinimumAbility>,
}

fn spell_fields(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: Option<&EntityTypeProfile>,
) -> SpellFields {
    let base = spell_levels_base(entity, profile);
    SpellFields {
        budget: spell_levels_budget(base, entity, ruleset),
        profile_base: profile.map(|p| p.spell_levels).unwrap_or(0),
        bonus: spell_levels_bonus(entity, ruleset),
        life_stage: life_stage_spell_levels(entity, ruleset),
        used: spell_levels_used(entity, ruleset),
        // The per-Te/Fo cap only matters on the Spells tab, which A2 will show
        // for any Hermetically trained entity — profile (a real magus) or
        // selection (the Abandoned Apprentice, D56) — so the gate reads the
        // union, not the bare profile flag. See
        // `docs/vf-audit/design-a0-is-magus-split.md` § 4 row 14'.
        level_caps: if is_hermetically_trained(entity, ruleset, profile) {
            spell_level_caps(entity, ruleset)
        } else {
            Vec::new()
        },
        // Magus-gated inside the engine already (the checklist is empty for anyone
        // the Order does not admit), so this needs no `is_magus` test of its own.
        minimum_abilities: magus_minimum_abilities(entity, ruleset),
    }
}

/// The Warping slice of [`EffectiveScores`]: the unified score/points plus the
/// off-budget grants a non-magus owes from them.
struct WarpingFields {
    score: u8,
    points: u32,
    owed_grants: Vec<Grant>,
}

fn warping_fields(entity: &Entity, ruleset: &Ruleset) -> WarpingFields {
    let totals = warping(entity, ruleset);
    WarpingFields {
        score: totals.score,
        points: totals.points,
        owed_grants: warping_owed_grants(entity, ruleset),
    }
}

/// The engine's Reputation grants, one UI slot each — the field rename
/// (`reputation_type` → `kind`) and nothing else.
///
/// This used to FLATTEN a player-chosen-kind grant (Famous) into one entry per
/// Reputation type, which offered four slots where `validate_reputations`
/// allows exactly one. The wildcard now travels as `kind: None` and the panel
/// renders a single row with a type `<select>`, so the offered slots and the
/// legal slots are the same count.
fn reputation_grants_for_ui(entity: &Entity, ruleset: &Ruleset) -> Vec<ReputationGrant> {
    reputation_grants(entity, ruleset)
        .into_iter()
        .map(|grant| ReputationGrant {
            source: grant.source,
            kind: grant.reputation_type,
            score: grant.score,
            max_score: grant.max_score,
        })
        .collect()
}

/// The Ability/Art/Characteristic slice of [`EffectiveScores`]: every bonus,
/// cap, floor, and derived readout that comes off the character's
/// Characteristics and their Virtue/Flaw modifiers, with no XP or spell
/// involvement.
struct CharacteristicFields {
    ability_bonuses: Vec<AbilityBonus>,
    art_bonuses: Vec<ArtBonus>,
    caps: BTreeMap<Characteristic, i32>,
    floors: BTreeMap<Characteristic, i32>,
    points_granted: i32,
    ability_score_floors: Vec<AbilityFloor>,
    size: i32,
    bonuses: Vec<CharacteristicBonus>,
    effective: BTreeMap<Characteristic, i32>,
    aging_drops: BTreeMap<Characteristic, u32>,
}

fn characteristic_fields(entity: &Entity, ruleset: &Ruleset) -> CharacteristicFields {
    CharacteristicFields {
        ability_bonuses: ability_bonuses(entity, ruleset),
        art_bonuses: art_bonuses(entity, ruleset),
        caps: characteristic_caps(ruleset),
        floors: characteristic_floors(ruleset),
        points_granted: characteristic_points_granted(entity, ruleset),
        ability_score_floors: ability_score_floors(entity, ruleset),
        size: size(entity, ruleset),
        bonuses: characteristic_bonuses(entity, ruleset),
        effective: effective_characteristics(entity, ruleset),
        aging_drops: characteristic_aging_drops(entity, ruleset),
    }
}

/// The Confidence / Gift-slot slice of [`EffectiveScores`]. Grouped because
/// Confidence and the Gift's free Supernatural-Ability slots share the same
/// "type default, or zero with no profile" shape.
///
/// It also carried the raw age→Ability cap until V1 (full-audit round 4) removed
/// it: no frontend surface ever read it, and the cap the engine actually enforces
/// is the per-ability `effective/reputation_and_caps.rs::ability_age_cap` — now
/// the single D29 resolution point, folding the age band with every override
/// (Foreign Upbringing's locality-dependent halving, the Affinity +2, and
/// Mentored by Demons' full waiver, F-194). Should the Ability spinner ever
/// grey scores past the cap, it must surface THAT one as a keyed map, the way
/// `spell_level_caps` already does for spells — not a single figure that
/// disagrees with the validator for exactly those Abilities.
struct ConfidenceFields {
    confidence_score: u8,
    confidence_points: u8,
    supernatural_free_total: u8,
    supernatural_free_used: u8,
}

fn confidence_fields(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: Option<&EntityTypeProfile>,
) -> ConfidenceFields {
    // Confidence is derived (type default + V/F); 0/0 when there is no profile.
    let derived_confidence = profile
        .map(|p| confidence(p.confidence_score, p.confidence_points, entity, ruleset))
        .unwrap_or(Confidence {
            score: 0,
            points: 0,
        });
    let supernatural_free = profile
        .map(|p| supernatural_free_slots(entity, ruleset, p))
        .unwrap_or(SupernaturalFreeSlots { total: 0, used: 0 });
    ConfidenceFields {
        confidence_score: derived_confidence.score,
        confidence_points: derived_confidence.points,
        supernatural_free_total: supernatural_free.total,
        supernatural_free_used: supernatural_free.used,
    }
}

/// The granted-selections / Virtue-Flaw-balance slice of [`EffectiveScores`]:
/// the effective budget ceilings plus the points actually spent, so the
/// balance bar's two halves come off one call each to the same engine module
/// (`validation::balance`).
struct GrantBudgetFields {
    granted_selections: Vec<Selection>,
    virtue_budget: u32,
    flaw_budget: u32,
    virtue_points: i32,
    flaw_points: i32,
}

fn grant_budget_fields(entity: &Entity, ruleset: &Ruleset) -> GrantBudgetFields {
    let ceilings = effective_point_ceilings(entity, ruleset).unwrap_or(PointCeilings {
        virtue_ceiling: 0,
        flaw_ceiling: 0,
    });
    let balance = compute_balance(entity, ruleset);
    GrantBudgetFields {
        granted_selections: entity_grants(entity, ruleset),
        virtue_budget: ceilings.virtue_ceiling,
        flaw_budget: ceilings.flaw_ceiling,
        virtue_points: balance.virtue_points,
        flaw_points: balance.flaw_points,
    }
}

/// The Decrepitude / True Faith / enchanted-item-level slice of
/// [`EffectiveScores`] — three unrelated single-number derived totals grouped
/// only because none has enough surface area to justify its own function.
struct DecrepitudeFaithItemFields {
    decrepitude_score: u8,
    true_faith_score: u8,
    item_level_budget: u32,
    item_level_used: u32,
}

fn decrepitude_faith_item_fields(entity: &Entity, ruleset: &Ruleset) -> DecrepitudeFaithItemFields {
    DecrepitudeFaithItemFields {
        decrepitude_score: decrepitude_score(entity, ruleset),
        true_faith_score: true_faith(entity, ruleset),
        item_level_budget: item_level_budget(entity, ruleset),
        item_level_used: item_level_used(entity),
    }
}

/// The Spell Mastery slice of [`EffectiveScores`].
struct SpellMasteryFields {
    xp: u32,
    floor: u8,
    advancement_affinity: Option<[u8; 2]>,
}

fn spell_mastery_fields(entity: &Entity, ruleset: &Ruleset) -> SpellMasteryFields {
    SpellMasteryFields {
        xp: spell_mastery_xp(entity, ruleset),
        floor: spell_mastery_floor(entity, ruleset),
        advancement_affinity: spell_mastery_advancement_affinity(entity, ruleset)
            .map(|(num, den)| [num, den]),
    }
}

/// The supernatural-being Might/power-level slice of [`EffectiveScores`].
struct MightPowerFields {
    might: Option<MightScore>,
    power_levels_budget: u32,
    power_levels_used: u32,
    focus_points_budget: u32,
    focus_points_used: u32,
}

fn might_power_fields(entity: &Entity, ruleset: &Ruleset) -> MightPowerFields {
    MightPowerFields {
        might: effective_might(entity, ruleset),
        power_levels_budget: power_levels_budget(entity, ruleset),
        power_levels_used: powers_used(entity),
        focus_points_budget: focus_points_budget(entity, ruleset),
        focus_points_used: focus_points_used(entity),
    }
}

/// Computes the score effects for `entity` against a loaded ruleset.
///
/// Pure field-by-field assembly: every group of related fields is computed by
/// its own helper above (mirroring [`EffectiveScores`]'s own doc-comment
/// groups), so this function does no branching of its own — it only wires
/// each group's output to the DTO's fields.
pub fn effective_scores_loaded(entity: &Entity, ruleset: &Ruleset) -> EffectiveScores {
    let profile = ruleset.profile(&entity.type_id);

    let characteristics = characteristic_fields(entity, ruleset);
    let xp = xp_fields(entity, ruleset);
    let spell = spell_fields(entity, ruleset, profile);
    let confidence = confidence_fields(entity, ruleset, profile);
    let grants = grant_budget_fields(entity, ruleset);
    let warping = warping_fields(entity, ruleset);
    let legacy_totals = decrepitude_faith_item_fields(entity, ruleset);
    let mastery = spell_mastery_fields(entity, ruleset);
    let might_power = might_power_fields(entity, ruleset);

    EffectiveScores {
        ability_bonuses: characteristics.ability_bonuses,
        art_bonuses: characteristics.art_bonuses,
        characteristic_caps: characteristics.caps,
        characteristic_floors: characteristics.floors,
        // A ruleset that declares no Characteristic table prices nothing, so zero
        // is the honest answer rather than a panic.
        characteristic_points_used: ruleset
            .characteristic_rules()
            .map_or(0, |rules| rules.total_cost(&entity.characteristics)),

        xp_total_demand: xp.total_demand,
        xp_general_used: xp.general_used,
        xp_general_pool: xp.general_pool,
        xp_general_bonus: xp.general_bonus,
        xp_max_flow: xp.max_flow,
        restricted_xp_pools: xp.restricted,
        life_stage: xp.life_stage,

        aging: aging_readout(entity, ruleset),

        characteristic_points_granted: characteristics.points_granted,
        ability_score_floors: characteristics.ability_score_floors,
        size: characteristics.size,
        characteristic_bonuses: characteristics.bonuses,
        characteristic_effective: characteristics.effective,
        characteristic_aging_drops: characteristics.aging_drops,

        granted_selections: grants.granted_selections,
        virtue_budget: grants.virtue_budget,
        flaw_budget: grants.flaw_budget,
        virtue_points: grants.virtue_points,
        flaw_points: grants.flaw_points,

        spell_levels_budget: spell.budget,
        spell_levels_profile_base: spell.profile_base,
        spell_levels_bonus: spell.bonus,
        spell_levels_life_stage: spell.life_stage,
        spell_levels_used: spell.used,
        spell_level_caps: spell.level_caps,
        magus_minimum_abilities: spell.minimum_abilities,

        confidence_score: confidence.confidence_score,
        confidence_points: confidence.confidence_points,
        supernatural_free_total: confidence.supernatural_free_total,
        supernatural_free_used: confidence.supernatural_free_used,

        reputation_grants: reputation_grants_for_ui(entity, ruleset),

        warping_score: warping.score,
        warping_points: warping.points,
        warping_owed_grants: warping.owed_grants,

        decrepitude_score: legacy_totals.decrepitude_score,
        true_faith_score: legacy_totals.true_faith_score,
        item_level_budget: legacy_totals.item_level_budget,
        item_level_used: legacy_totals.item_level_used,

        spell_mastery_xp: mastery.xp,
        spell_mastery_floor: mastery.floor,
        spell_mastery_advancement_affinity: mastery.advancement_affinity,

        might: might_power.might,
        power_levels_budget: might_power.power_levels_budget,
        power_levels_used: might_power.power_levels_used,
        focus_points_budget: might_power.focus_points_budget,
        focus_points_used: might_power.focus_points_used,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arm_rules::Effect;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::ruleset_io::load_ruleset_from_dir;

    /// The repository root — `crates/arm-app` is two levels below it.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The Spell Mastery Advancement Affinity is authored rules data: an
    /// arbitrary "counts as `num`/`den` of itself" ratio on
    /// [`Effect::GrantsSpellMastery`], which the engine prices through the
    /// general `effective/xp.rs::charged_cost`. The IPC boundary must carry that
    /// ratio rather than a "is there one at all" boolean — collapsing it leaves
    /// the frontend re-expanding the number as a literal, so a Virtue authoring
    /// any other ratio would be a data-only change that silently produces a
    /// wrong total on screen (full-audit round 2, V2).
    ///
    /// The expectation is read back out of the shipped catalogue rather than
    /// written as a literal, so this asserts "whatever ratio the data authors
    /// arrives intact" and never pins the catalogue to one value.
    #[test]
    fn the_mastery_affinity_reaches_the_frontend_as_the_authored_ratio() {
        let localized = load_ruleset_from_dir(&repo_root().join("rules"), "en").unwrap();
        let json = fs::read_to_string(repo_root().join("examples/magus_sample.json")).unwrap();
        let mut entity: Entity = serde_json::from_str(&json).unwrap();
        let granting = Id::new("virtue.flawless_magic");
        entity.selections.push(Selection::new(granting.clone()));

        let authored = localized
            .ruleset
            .item(&granting)
            .expect("the shipped catalogue must still carry the mastery-granting Virtue")
            .effects
            .iter()
            .find_map(|effect| match effect {
                Effect::GrantsSpellMastery {
                    advancement_num,
                    advancement_den,
                    ..
                } => Some([*advancement_num, *advancement_den]),
                _ => None,
            })
            .expect("virtue.flawless_magic must still grant Spell Mastery");

        assert_eq!(
            spell_mastery_fields(&entity, &localized.ruleset).advancement_affinity,
            Some(authored),
            "the DTO must carry the authored (num, den), not a boolean"
        );
    }

    /// K1 (round-2 CRITICAL): `xp_fields` (behind `effective_scores_loaded`,
    /// which the `effective_scores` Tauri command calls on every File → Open)
    /// used to call the raw `xp_allocation` directly with no flow-solve
    /// node-count check, reaching its internal `assert!` for a save whose
    /// `ability_scores` exceeds `MAX_XP_SOLVE_NODES` — a panic on a plain
    /// File → Open of a hostile save, racing the friendly
    /// `CODE_XP_SOLVE_BOUND_EXCEEDED` the concurrent `validate_entity` call
    /// would otherwise produce. Must degrade to all-zero/empty XP fields
    /// instead.
    #[test]
    fn effective_scores_loaded_degrades_the_xp_fields_over_the_solve_bound() {
        use arm_rules::{AbilityScore, EntityKind, RulesetRef, RulesetSources};

        // The engine requires at least one V/F category-tagged "personality"
        // item once a catalogue is shipped at all.
        const ITEMS: &str = r#"[
          { "id": "flaw.filler", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
        ]"#;
        const TYPES: &str = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general"], "creation_phases": [] }
        ]"#;
        const ABILITIES: &str = r#"{
          "advancement": [{ "score": 1, "total_xp": 5 }],
          "abilities": [{ "id": "ability.artes_liberales", "category": "general" }]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap();

        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        e.xp_pool = 1_000_000;
        // Comfortably past MAX_XP_SOLVE_NODES: rejected before the solve runs,
        // so this stays fast regardless of count. Deliberately not a boundary
        // fixture — the bound is `pub(crate)` in `arm-rules`, so this crate
        // cannot name it, and pinning the exact edge against a literal here
        // would be a second copy of the number that nothing keeps in step. The
        // boundary itself is covered where the constant lives, in
        // `effective/xp.rs::xp_solve_scale`'s own tests; what this test owns is
        // the app-layer consequence — far over the bound must surface as zeroed
        // totals rather than a panic.
        e.ability_scores = (0..2049)
            .map(|_| AbilityScore {
                ability: Id::new("ability.artes_liberales"),
                parameter: None,
                score: 1,
                specialty: None,
            })
            .collect();

        let scores = effective_scores_loaded(&e, &rs);
        assert_eq!(scores.xp_total_demand, 0);
        assert_eq!(scores.xp_general_used, 0);
        assert_eq!(scores.xp_general_pool, 0);
        assert_eq!(scores.xp_general_bonus, 0);
        assert_eq!(scores.xp_max_flow, 0);
        assert!(scores.restricted_xp_pools.is_empty());
    }

    /// Row 14' of `docs/vf-audit/design-a0-is-magus-split.md` § 4: this DTO's
    /// per-Te/Fo spell-level-cap list was gated on the bare
    /// `profile.hermetically_trained` flag, so an entity carrying
    /// `Effect::ConfersHermeticTraining` on a non-magus profile (the
    /// Abandoned-Apprentice shape) got an empty list — no Spells tab data —
    /// even though D56 says he must have Casting/Lab totals same as a real
    /// magus. Switching the gate to `is_hermetically_trained` (the
    /// profile-OR-selection union) fixes it. Test-fixture-only, per the design
    /// note's sub-slice ordering: the shipped `flaw.abandoned_apprentice` entry
    /// is not touched until slice D3.
    #[test]
    fn spell_fields_populates_level_caps_for_a_trained_non_magus_test_fixture() {
        use arm_rules::{EntityKind, RulesetRef, RulesetSources};

        const ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.test_confers_training", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "confers_hermetic_training" }] }
        ]"#;
        const TYPES: &str = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "personality"],
            "hermetically_trained": false, "order_member": false, "creation_phases": [] }
        ]"#;
        const ARTS: &str = r#"{
          "advancement": [{ "score": 1, "total_xp": 1 }],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            arts: Some(ARTS),
            ..RulesetSources::default()
        })
        .unwrap();
        let profile = rs.profile(&Id::new("companion"));

        let mut untrained = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        untrained.selections = vec![Selection::new(Id::new("flaw.optimistic"))];
        assert!(
            spell_fields(&untrained, &rs, profile).level_caps.is_empty(),
            "an ordinary companion has no Spells tab"
        );

        let mut trained = untrained.clone();
        trained.selections = vec![Selection::new(Id::new("flaw.test_confers_training"))];
        assert!(
            !spell_fields(&trained, &rs, profile).level_caps.is_empty(),
            "a companion carrying a training-conferring selection must get spell-level caps \
             too, not just the magus profile"
        );
    }
}
