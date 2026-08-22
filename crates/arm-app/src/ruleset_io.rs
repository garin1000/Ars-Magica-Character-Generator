//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.

use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use arm_rules::validation::{aging_error_issue, childhood_rejection_issues};
use arm_rules::{
    AbilityBonus, AbilityFloor, AgingError, AgingNote, AgingOutcome, AgingTotal, AgingYearRequest,
    ArtBonus, Characteristic, CharacteristicBonus, Confidence, CrisisPreview, Entity, EntityKind,
    EntityTypeProfile, Grant, Id, LifeStageBudget, LocalizedRuleset, MagusMinimumAbility,
    MightScore, PointCeilings, ReputationType, RestrictedXpPool, Ruleset, RulesetSources,
    Selection, SpellLevelCap, SupernaturalFreeSlots, ValidationIssue, ValidationMode,
    ValidationResult, ability_bonuses, ability_score_floors, age_ability_cap, aging_schedule,
    aging_total, apply_childhood_package, art_bonuses, characteristic_aging_drops,
    characteristic_bonuses, characteristic_caps, characteristic_floors,
    characteristic_points_granted, checked_xp_allocation, compute_balance, confidence,
    decrepitude_score, effective_characteristics, effective_might, effective_point_ceilings,
    entity_grants, item_level_budget, item_level_used, life_stage_spell_levels, longevity_bonus,
    magus_minimum_abilities, power_levels_budget, powers_used, reputation_grants, resolve_outcome,
    resolve_year, revert_year, size, spell_level_caps, spell_levels_base, spell_levels_bonus,
    spell_levels_budget, spell_levels_used, spell_mastery_advancement_affinity,
    spell_mastery_floor, spell_mastery_xp, supernatural_free_slots, true_faith, validate, warping,
    warping_owed_grants,
};
use serde::Serialize;

use crate::error::AppError;

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
    /// Effective virtue/flaw point ceilings (base budget + Mythic Companion type
    /// bonus) so the balance bar shows the true budget (a Devil Child's 37/17,
    /// not the base 20/10). Budget numbers stay engine-authoritative.
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
    /// modifiers + the levels its post-Gauntlet years bought) — the "available" side
    /// of the spell-levels bar. The base is the per-character
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
    /// (Ars Magica - Definitive Edition (Core Rules).md:2471). 0 for a magus standing
    /// at its Gauntlet and for anyone who serves no apprenticeship.
    ///
    /// **Not a second budget** like apprenticeship's 120 levels (`:2435`), which are
    /// the type profile's `spell_levels` and reach the UI as
    /// [`Self::spell_levels_profile_base`] — these are added on top of it.
    pub spell_levels_life_stage: u32,
    /// The spell levels the chosen spells consume — the "used" side of the bar.
    pub spell_levels_used: u32,
    /// Per-Technique/Form maximum learnable spell level (Te + Fo + Int + Magic
    /// Theory + 3), so the spell picker greys a spell above the magus's cap
    /// without recomputing the derivation in JS. Empty for a non-magus (no Spells
    /// tab). Engine-authoritative; the UI only reads it.
    pub spell_level_caps: Vec<SpellLevelCap>,
    /// The Hermetic minimum-Ability checklist: what the Order demands (Core:2437) and
    /// what the rulebook recommends (Core:2451-2461), each with the character's bought
    /// score and whether it suffices. Empty for a non-magus, exactly like
    /// [`Self::spell_level_caps`] — `:2437` is about admission to the Order.
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
    /// The character's age → max-Ability-score cap (base, before Affinity's +2),
    /// surfaced so the UI shows one source of truth. `None` when age is unset.
    pub age_ability_cap: Option<u8>,
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
    /// Whether a Virtue doubles all Spell-Mastery Advancement Totals (Flawless
    /// Magic), halving the XP each mastery point costs — so the UI's mastery
    /// accounting charges the same reduced cost the engine does.
    pub spell_mastery_advancement_doubled: bool,
    /// The supernatural being's effective Might Score + Realm (base + same-Realm
    /// Virtue grants), or `None` for an ordinary character. Engine-authoritative.
    pub might: Option<MightScore>,
    /// Derived power-levels budget the being's Might Virtues grant (Demonic Blood
    /// 30, Demonic Powers +20); 0 when none — the "available" side of the bar.
    pub power_levels_budget: u32,
    /// The total power level the being's `powers` consume — the "used" side of the
    /// power-levels bar (engine-authoritative; the UI never recomputes it).
    pub power_levels_used: u32,
}

/// Everything about a character's aging that does **not** depend on a die.
///
/// The stress die is player input the entity must never store (`:16567` — the
/// engine has no `rand` dependency and never will), so the roll itself is a
/// command the player triggers. Every field here, by contrast, is a pure function
/// of `(entity, ruleset)`, which is exactly why it rides on the always-recomputed
/// effective-scores payload rather than on a separate request: it can never be
/// stale, and the UI never has to ask for it.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16563-16617.
#[derive(Debug, Clone, Serialize)]
pub struct AgingReadout {
    /// The first age at which a roll is owed — 36 under the shipped rules, the
    /// Winter after the character turns 35 (`:16565`).
    pub first_roll_age: u32,
    /// The age aging begins AFTER (`:16565`) — the ruleset's own `start_age`,
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
    /// ⌈age / divisor⌉ at the character's **actual** age (`:16577`); 0 when no
    /// age is entered, because there is then no age term to compute.
    pub age_modifier: i32,
    /// The Living Conditions modifier the total SUBTRACTS (`:16569`, `:16571`),
    /// the chosen rows and the Virtue/Flaw contributions together.
    pub living_conditions_modifier: i32,
    /// The Longevity Ritual modifier the total SUBTRACTS (`:16569`); 0 with no
    /// ritual, or with one whose bonus the player has not entered.
    pub longevity_modifier: i32,
    /// Whether the `:16575` clamp stands over this character — he holds a
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
    /// Blood's -1, `:3801`), which the book's three-line formula does not name
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16563-16617.
fn aging_readout(entity: &Entity, ruleset: &Ruleset) -> Option<AgingReadout> {
    let rules = ruleset.aging()?;
    let schedule = aging_schedule(entity, ruleset);
    // No age entered means no age term (`:16577` asks for the actual age, and
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
        longevity_clamp_active: rules
            .longevity_clamp
            .as_ref()
            .is_some_and(|clamp| age < clamp.until_age)
            && longevity_bonus(entity, ruleset).is_some(),
        // The probe's die was 0, so its uncapped total IS the non-die half.
        fixed_total: terms.uncapped_total,
    })
}

/// A Reputation a Virtue/Flaw authorizes the character to start with (the UI
/// pre-fills a new Reputation row from this; content is player-supplied).
#[derive(Debug, Clone, Serialize)]
pub struct ReputationGrant {
    pub kind: ReputationType,
    pub score: u8,
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
        // The per-Te/Fo cap only matters on the (magus-only) Spells tab, so it is
        // computed only for a magus — other types ship an empty list.
        level_caps: if profile.is_some_and(|p| p.is_magus) {
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

/// Flattens the engine's Reputation grants into one [`ReputationGrant`] per
/// concrete type: a player-chosen-kind grant (`kind == None`, e.g. Famous)
/// authorizes any type, so it becomes one add-control per Reputation type;
/// concrete-kind grants pass through unchanged. Validation still enforces the
/// single-slot count (see `validate_reputations`).
fn reputation_grants_for_ui(entity: &Entity, ruleset: &Ruleset) -> Vec<ReputationGrant> {
    reputation_grants(entity, ruleset)
        .into_iter()
        .flat_map(|grant| match grant.reputation_type {
            Some(kind) => vec![ReputationGrant {
                kind,
                score: grant.score,
            }],
            None => ReputationType::ALL
                .into_iter()
                .map(|kind| ReputationGrant {
                    kind,
                    score: grant.score,
                })
                .collect(),
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
        caps: characteristic_caps(entity, ruleset),
        floors: characteristic_floors(entity, ruleset),
        points_granted: characteristic_points_granted(entity, ruleset),
        ability_score_floors: ability_score_floors(entity, ruleset),
        size: size(entity, ruleset),
        bonuses: characteristic_bonuses(entity, ruleset),
        effective: effective_characteristics(entity, ruleset),
        aging_drops: characteristic_aging_drops(entity, ruleset),
    }
}

/// The Confidence / Gift-slot / age-cap slice of [`EffectiveScores`]. Grouped
/// because Confidence and the Gift's free Supernatural-Ability slots share the
/// same "type default, or zero with no profile" shape.
struct ConfidenceFields {
    confidence_score: u8,
    confidence_points: u8,
    supernatural_free_total: u8,
    supernatural_free_used: u8,
    age_ability_cap: Option<u8>,
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
        age_ability_cap: age_ability_cap(entity, ruleset),
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
    advancement_doubled: bool,
}

fn spell_mastery_fields(entity: &Entity, ruleset: &Ruleset) -> SpellMasteryFields {
    SpellMasteryFields {
        xp: spell_mastery_xp(entity, ruleset),
        floor: spell_mastery_floor(entity, ruleset),
        advancement_doubled: spell_mastery_advancement_affinity(entity, ruleset).is_some(),
    }
}

/// The supernatural-being Might/power-level slice of [`EffectiveScores`].
struct MightPowerFields {
    might: Option<MightScore>,
    power_levels_budget: u32,
    power_levels_used: u32,
}

fn might_power_fields(entity: &Entity, ruleset: &Ruleset) -> MightPowerFields {
    MightPowerFields {
        might: effective_might(entity, ruleset),
        power_levels_budget: power_levels_budget(entity, ruleset),
        power_levels_used: powers_used(entity),
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
        age_ability_cap: confidence.age_ability_cap,

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
        spell_mastery_advancement_doubled: mastery.advancement_doubled,

        might: might_power.might,
        power_levels_budget: might_power.power_levels_budget,
        power_levels_used: might_power.power_levels_used,
    }
}

/// The outcome of applying a Sample Childhood package: either the entity with the
/// package's rows written, or the reasons it could not be applied.
///
/// The reasons cross the IPC edge as ordinary [`ValidationIssue`]s so the frontend
/// renders them through the `issue-<code>` Fluent path it already has, and no
/// English prose ever crosses the boundary. That is also why a rejection is a
/// perfectly ordinary `Ok` outcome rather than an [`AppError`]: an unanswered slot
/// is a finding about the form the player just submitted, not a failure of the
/// command.
/// The entity is boxed so the two variants stay comparable in size (an `Entity` is
/// far larger than a list of issues); `Box<Entity>` serializes exactly as `Entity`
/// does, so the JSON the frontend sees is unaffected.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ChildhoodApplication {
    Applied { entity: Box<Entity> },
    Rejected { issues: Vec<ValidationIssue> },
}

/// Applies the Sample Childhood package `package` names to `entity` against a
/// loaded ruleset, localizing the engine's rejections on the way out.
///
/// The engine owns every decision here ([`apply_childhood_package`]): what the
/// package writes, that the write is a monotone raise, and what makes it
/// impossible. This only chooses the shape the frontend receives.
pub fn apply_childhood_package_loaded(
    entity: &Entity,
    package: &Id,
    slot_values: &BTreeMap<String, String>,
    ruleset: &Ruleset,
) -> ChildhoodApplication {
    match apply_childhood_package(entity, package, slot_values, ruleset) {
        Ok(entity) => ChildhoodApplication::Applied {
            entity: Box::new(entity),
        },
        Err(rejections) => ChildhoodApplication::Rejected {
            issues: childhood_rejection_issues(&rejections, ruleset),
        },
    }
}

/// The outcome of previewing one aging roll: the reading, or the reason there was
/// none.
///
/// The three aging commands are shaped on [`ChildhoodApplication`] throughout: a
/// refusal is an ordinary `Ok` outcome carrying localizable
/// [`ValidationIssue`]s, never an [`AppError`], because a die typed against a year
/// already rolled is a finding about the form the player just submitted rather
/// than a failure of the command. [`arm_rules::AgingError`] is plain data with no
/// prose of its own, so `aging_error_issue` turns it into something the frontend
/// renders through the `issue-<code>` path it already has, and no English string
/// crosses the boundary.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingProjection {
    Previewed {
        total: AgingTotal,
        outcome: AgingOutcome,
        /// The Crisis this year would send the character to, read whole and
        /// **written nowhere** — present only once the row demands one, the
        /// player has thrown the Simple Die, and the year is one
        /// [`arm_rules::resolve_year`] would accept. See
        /// [`aging_preview_loaded`] for why the last condition is not optional.
        ///
        /// Boxed to keep the two variants comparable in size;
        /// `Box<CrisisPreview>` serializes exactly as `CrisisPreview` does.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        crisis: Option<Box<CrisisPreview>>,
    },
    Rejected {
        issues: Vec<ValidationIssue>,
    },
}

/// The outcome of applying one aging roll: the character it made plus the reading
/// that made it, or the reason it was refused.
///
/// The reading comes back with the entity so the UI can confirm what it applied
/// without recomputing it against a character that has since changed. The entity
/// is boxed so the two variants stay comparable in size; `Box<Entity>` serializes
/// exactly as `Entity` does.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingApplication {
    Applied {
        entity: Box<Entity>,
        total: AgingTotal,
        outcome: AgingOutcome,
        /// The Crisis the year sent the character to, read whole — present only
        /// when the row demanded one *and* the player had thrown the Simple Die.
        /// The log records the same figures, but only from here can the UI show
        /// what surviving it would take, which the log has no room for.
        ///
        /// Boxed for the reason [`Self::Applied::entity`] is — it keeps the two
        /// variants comparable in size — and `Box<CrisisPreview>` serializes
        /// exactly as `CrisisPreview` does.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        crisis: Option<Box<CrisisPreview>>,
        /// What the year changed that the character itself cannot show — today,
        /// only the Longevity Ritual a Crisis spends (`:16573`). Empty for almost
        /// every year, and each variant is rendered through Fluent by the caller.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        notes: Vec<AgingNote>,
    },
    Rejected {
        issues: Vec<ValidationIssue>,
    },
}

/// The outcome of taking one applied aging roll back off.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingReversion {
    Reverted { entity: Box<Entity> },
    Rejected { issues: Vec<ValidationIssue> },
}

/// Reads one year's aging roll without writing anything: the AGING TOTAL the typed
/// `die` makes at `age`, and the row it lands on.
///
/// Read-only on purpose. The die is player input the entity must never store — the
/// engine has no `rand` dependency and a stress die explodes, so no lookup table
/// could stand in for this — which is why the calculator asks the engine instead of
/// keeping the number on the character.
///
/// # The Crisis rides here, and there is no second command
///
/// A Crisis is not a second question about a second thing: it exists only because
/// this year's row demanded one (`:16602`, `:16611`), and its total counts the
/// Decrepitude this very year raised. A command of its own would let the frontend
/// pair a CRISIS TOTAL with an aging outcome that no longer calls for one, which is
/// exactly the pairing `crisis_preview` was composed to take out of a caller's
/// hands.
///
/// # Why it resolves the year rather than reading the character
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on the
/// > Crisis Table. (`:16619`)
///
/// The Aging Points the row awards ARE that increase, so a Crisis read off the
/// character *standing in front of you* is one Decrepitude short of the one the
/// year writes — the player would be shown 14 and then watch the log record 15. So
/// this builds the very [`AgingYearRequest`] the Apply would send, resolves it in
/// memory, and keeps only the reading: same request, same answer, and the entity
/// it made is dropped on the spot. Nothing is written, and nothing can drift.
///
/// A year the engine would refuse — an unplaced distribution, a year already
/// recorded — yields no crisis reading, while the AGING TOTAL and the outcome still
/// stand. The player has to be told a Crisis follows and what to place *before* he
/// can place it, which is the order `:16619` itself asks for.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16615, :16619.
pub fn aging_preview_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> AgingProjection {
    let reading = aging_total(entity, ruleset, age, die)
        .and_then(|total| resolve_outcome(entity, ruleset, total.total).map(|out| (total, out)));
    match reading {
        Some((total, outcome)) => AgingProjection::Previewed {
            total,
            outcome,
            crisis: previewed_crisis(entity, ruleset, age, die, distribution, crisis_die),
        },
        // The one thing that can be missing is the aging block itself; every other
        // input is the character's own.
        None => AgingProjection::Rejected {
            issues: vec![aging_error_issue(&AgingError::NoAgingRules)],
        },
    }
}

/// The Crisis one previewed year would produce, read off the character the year
/// would make rather than the one it started from (`:16619`).
///
/// The whole of it is [`resolve_year`] run for its reading alone: the entity it
/// returns is dropped, so this stays as read-only as the preview it serves while
/// remaining, by construction, the same answer the Apply will give. A refusal is
/// simply no reading — the calculator's own findings already say why.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16619.
fn previewed_crisis(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> Option<Box<CrisisPreview>> {
    let request = AgingYearRequest {
        age,
        die,
        distribution: distribution.clone(),
        crisis_die: Some(crisis_die?),
    };
    resolve_year(entity, ruleset, &request)
        .ok()?
        .crisis
        .map(Box::new)
}

/// Applies one year's aging roll, returning the character it makes.
///
/// The engine owns every decision ([`resolve_year`] is the aging subsystem's
/// single writer): what the row awards, where the points may go, whether the
/// apparent age advances, and what makes the year impossible. This only chooses
/// the shape the frontend receives.
///
/// `crisis_die` is the **Simple Die** the player threw at the Crisis Table
/// (`:16621`), or `None` for a Crisis nobody has rolled yet — a legitimate state,
/// because the aging roll happened whether or not the second die was thrown. A die
/// given for a year the table sent to no Crisis is simply unused: whether a Crisis
/// happened is `:16602`/`:16611`'s call, never the player's.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16621.
pub fn aging_apply_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> AgingApplication {
    let request = AgingYearRequest {
        age,
        die,
        distribution: distribution.clone(),
        crisis_die,
    };
    match resolve_year(entity, ruleset, &request) {
        Ok(result) => AgingApplication::Applied {
            entity: Box::new(result.entity),
            total: result.total,
            outcome: result.outcome,
            crisis: result.crisis.map(Box::new),
            notes: result.notes,
        },
        Err(error) => AgingApplication::Rejected {
            issues: vec![aging_error_issue(&error)],
        },
    }
}

/// Takes one applied aging year back off, exactly.
///
/// Addresses the year by `age`, which is what the log entry records; a
/// hand-written free-text entry carries none and is out of reach here by
/// construction (the ordinary log editor removes it instead).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16577-16617.
pub fn aging_revert_loaded(entity: &Entity, ruleset: &Ruleset, age: u32) -> AgingReversion {
    match revert_year(entity, ruleset, age) {
        Ok(entity) => AgingReversion::Reverted {
            entity: Box::new(entity),
        },
        Err(error) => AgingReversion::Rejected {
            issues: vec![aging_error_issue(&error)],
        },
    }
}

/// File extension for a saved entity of the given kind: `armc` for characters,
/// `armcov` for covenants ("Ars Magica character/covenant"). They are plain JSON
/// underneath, but a distinct extension lets the OS associate and filter them.
pub fn entity_extension(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "armc",
        EntityKind::Covenant => "armcov",
    }
}

/// Extension-free base file name for a kind, shared by the save default and the
/// Markdown-export default so the two can never drift apart. Deliberately not the
/// entity's own `name`: that is arbitrary user text and would need per-platform
/// filename sanitization to be safe, for no gain — the dialog lets the user type
/// whatever name they want.
fn entity_base_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "character",
        EntityKind::Covenant => "covenant",
    }
}

/// Default save file name for a kind, e.g. `character.armc`.
pub fn default_file_name(kind: EntityKind) -> String {
    format!("{}.{}", entity_base_name(kind), entity_extension(kind))
}

/// Extension of an exported Markdown character sheet.
pub const MARKDOWN_EXTENSION: &str = "md";

/// Default file name for a Markdown export, e.g. `character.md`.
pub fn default_markdown_file_name(kind: EntityKind) -> String {
    format!("{}.{MARKDOWN_EXTENSION}", entity_base_name(kind))
}

/// Prefill file name for the Markdown-export dialog: the document's own save file
/// re-extensioned, so `gerhard.armc` exports as `gerhard.md`. Falls back to the
/// kind default for a document that was never saved, or a path with no usable base
/// name — the entity's `name` stays out of it for the reason
/// [`entity_base_name`] documents.
pub fn markdown_file_name_for(current_path: Option<&str>, kind: EntityKind) -> String {
    let base = current_path
        .and_then(|path| path.rsplit(['/', '\\']).next())
        .unwrap_or_default();
    // Split at the LAST dot, but only past the first byte: a base name starting
    // with the dot (`.armc`) is nothing but an extension and leaves no stem.
    let stem = match base.rfind('.') {
        Some(dot) if dot > 0 => &base[..dot],
        Some(_) => "",
        None => base,
    };
    if stem.is_empty() {
        return default_markdown_file_name(kind);
    }
    format!("{stem}.{MARKDOWN_EXTENSION}")
}

/// Directory the document's save file sits in, used as the export dialog's starting
/// folder so the export is offered next to the character file. `None` when the path
/// names no directory at all (a bare file name, or a document never saved).
pub fn save_file_directory(current_path: Option<&str>) -> Option<&str> {
    let path = current_path?;
    let separator = path.rfind(['/', '\\'])?;
    // The separator is kept, so a root-level file yields "/" rather than "".
    Some(&path[..=separator])
}

/// Appends `ext` when the chosen path has no extension, so a user who types just
/// "testchar" still gets "testchar.armc". An explicit extension (`.armc`,
/// `.json`, …) the user typed is respected.
pub fn ensure_extension(path: PathBuf, ext: &str) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension(ext)
    } else {
        path
    }
}

/// Stable ID + version of the shipped ruleset. These are slug-style identifiers,
/// not user-facing text, so they live in code rather than Fluent.
pub const RULESET_ID: &str = "arm5-core";
pub const RULESET_VERSION: &str = "2024.1";

/// Given ordered candidate rules directories, returns the first one that
/// actually holds the rules data, or `None` when none do.
///
/// Tauri's `BaseDirectory::Resource` does not resolve to the executable's own
/// directory for a portable Linux build: `resource_dir` there falls back to a
/// system path (`/usr/lib/<name>`) that a portable extract never populates. So
/// the command offers both the resource path and the directory next to the
/// executable as candidates and lets this pick whichever is real. A candidate is
/// considered valid when it contains `core/character_types.json`, a required
/// rules file.
pub fn pick_rules_dir(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|dir| dir.join("core/character_types.json").is_file())
        .cloned()
}

/// Loads the shipped ruleset for `lang` from a rules directory laid out as
/// `core/*.json` + `i18n/<lang>/*.json`, parsing and integrity-checking it via
/// the engine. Returns the ruleset paired with localized display text.
pub fn load_ruleset_from_dir(rules_dir: &Path, lang: &str) -> Result<LocalizedRuleset, AppError> {
    let point_items_json = fs::read_to_string(rules_dir.join("core/virtues_flaws.json"))?;
    let type_profiles_json = fs::read_to_string(rules_dir.join("core/character_types.json"))?;
    let abilities_json = fs::read_to_string(rules_dir.join("core/abilities.json"))?;
    let arts_json = fs::read_to_string(rules_dir.join("core/arts.json"))?;
    let houses_json = fs::read_to_string(rules_dir.join("core/houses.json"))?;
    let mythic_types_json = fs::read_to_string(rules_dir.join("core/mythic_companion_types.json"))?;
    let spells_json = fs::read_to_string(rules_dir.join("core/spells.json"))?;
    let spell_mastery_abilities_json =
        fs::read_to_string(rules_dir.join("core/spell_mastery_abilities.json"))?;
    let equipment_json = fs::read_to_string(rules_dir.join("core/equipment.json"))?;
    let characteristics_json = fs::read_to_string(rules_dir.join("core/characteristics.json"))?;
    let life_stages_json = fs::read_to_string(rules_dir.join("core/life_stages.json"))?;
    let childhoods_json = fs::read_to_string(rules_dir.join("core/childhoods.json"))?;
    let aging_json = fs::read_to_string(rules_dir.join("core/aging.json"))?;

    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &point_items_json,
        type_profiles: &type_profiles_json,
        abilities: Some(&abilities_json),
        arts: Some(&arts_json),
        houses: Some(&houses_json),
        mythic_types: Some(&mythic_types_json),
        spells: Some(&spells_json),
        spell_mastery_abilities: Some(&spell_mastery_abilities_json),
        equipment: Some(&equipment_json),
        // An empty characteristics file means the ruleset ships no characteristic
        // rules (the `Option` is the engine's honest "absent" signal).
        characteristics: (!characteristics_json.is_empty())
            .then_some(characteristics_json.as_str()),
        // Likewise: an empty life-stages file means the ruleset ships no life
        // stages, leaving `Entity::xp_pool` the only source of experience.
        life_stages: (!life_stages_json.is_empty()).then_some(life_stages_json.as_str()),
        // And likewise: an empty Sample Childhood file means the ruleset offers no
        // ready-made packages, leaving the childhood blocks to be divided by hand —
        // which the rules explicitly allow ("you can spend the 45 experience points
        // for yourself, as well",
        // Ars Magica - Definitive Edition (Core Rules).md:2382).
        childhoods: (!childhoods_json.is_empty()).then_some(childhoods_json.as_str()),
        // And likewise: an empty aging file means the ruleset ships no aging
        // tables, which stands the aging subsystem down rather than letting the
        // engine invent a table.
        aging: (!aging_json.is_empty()).then_some(aging_json.as_str()),
    })?;
    // Load the requested language's rules text. For any non-English language,
    // English is loaded as a per-field fallback so a not-yet-translated string
    // (e.g. a missing German spell description) surfaces the English text rather
    // than rendering empty. English needs no fallback (it is the source of truth).
    let i18n = read_i18n_sources(rules_dir, lang)?;
    let i18n_refs: Vec<&str> = i18n.iter().map(String::as_str).collect();
    let localized = if lang == "en" {
        LocalizedRuleset::from_merged(ruleset, &i18n_refs)?
    } else {
        let fallback = read_i18n_sources(rules_dir, "en")?;
        let fallback_refs: Vec<&str> = fallback.iter().map(String::as_str).collect();
        LocalizedRuleset::from_merged_with_fallback(ruleset, &i18n_refs, &fallback_refs)?
    };
    Ok(localized)
}

/// Reads the ten `i18n/<lang>/*.json` rules-text files for a language, in the
/// stable domain order the localized ruleset merges them. A missing file is an
/// error (each language ships the full set), surfaced to the caller.
fn read_i18n_sources(rules_dir: &Path, lang: &str) -> Result<Vec<String>, AppError> {
    const FILES: [&str; 10] = [
        "virtues_flaws.json",
        "abilities.json",
        "arts.json",
        "houses.json",
        "mythic_companion_types.json",
        "spells.json",
        "spell_mastery_abilities.json",
        "equipment.json",
        "childhoods.json",
        "aging.json",
    ];
    FILES
        .iter()
        .map(|file| {
            fs::read_to_string(rules_dir.join(format!("i18n/{lang}/{file}")))
                .map_err(AppError::from)
        })
        .collect()
}

/// Validates an entity against a loaded ruleset and applies the caller's mode
/// (Enforced keeps errors, Advisory downgrades them to warnings, Silent clears).
pub fn validate_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    mode: ValidationMode,
) -> ValidationResult {
    validate(entity, ruleset).apply_mode(mode)
}

/// Writes an entity to `path` as canonical, pretty JSON. The entity is
/// normalized first (sorting selections and parameters) so the output is
/// byte-stable for zero-noise git diffs — the engine no longer sorts implicitly
/// on serialize.
pub fn save_entity_to_path(entity: &Entity, path: &Path) -> Result<(), AppError> {
    let mut canonical = entity.clone();
    // Stamp the current schema version so app-written saves never drift from the
    // engine's `SCHEMA_VERSION` (a save always reflects the shape it was written by).
    canonical.schema_version = arm_rules::SCHEMA_VERSION;
    canonical.normalize();
    let json = serde_json::to_string_pretty(&canonical)?;
    fs::write(path, json)?;
    Ok(())
}

/// Renders `entity` as Markdown and writes it to `path`.
///
/// `ruleset` is the cached localized ruleset that supplies item display names, and
/// `labels` the frontend's localized document chrome (see
/// [`arm_rules::export::LABEL_KEYS`]) — no user-facing string originates here.
///
/// The ruleset is an `Option` rather than a reference because "no ruleset loaded"
/// is a real outcome the export must report: without it not one display name can be
/// resolved, so it fails with [`AppError::NotLoaded`] and writes nothing. Deciding
/// that here rather than in the command shim keeps the case testable without a
/// Tauri runtime.
pub fn export_markdown_to_path(
    entity: &Entity,
    ruleset: Option<&LocalizedRuleset>,
    labels: &BTreeMap<String, String>,
    path: &Path,
) -> Result<(), AppError> {
    let ruleset = ruleset.ok_or(AppError::NotLoaded)?;
    let markdown = arm_rules::character_markdown(entity, ruleset, labels)?;
    fs::write(path, markdown)?;
    Ok(())
}

/// Reads and deserializes an entity from `path`, applying save migrations.
///
/// A pre-schema-10 save's manual `aging_reductions` are folded into `aging_points`
/// (aging drops are now derived); the migration is logged so a stale save is
/// visibly upgraded on load. See [`arm_rules::load_entity_migrating`].
pub fn load_entity_from_path(path: &Path) -> Result<Entity, AppError> {
    let json = fs::read_to_string(path)?;
    let loaded = arm_rules::load_entity_migrating(&json)?;
    if !loaded.migrated_aging_characteristics.is_empty() {
        let characteristics: Vec<String> = loaded
            .migrated_aging_characteristics
            .iter()
            .map(|c| c.to_string())
            .collect();
        eprintln!(
            "save migration: folded legacy aging_reductions into aging_points for {} ({})",
            path.display(),
            characteristics.join(", ")
        );
    }
    Ok(loaded.entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_and_default_name_per_kind() {
        assert_eq!(entity_extension(EntityKind::Character), "armc");
        assert_eq!(entity_extension(EntityKind::Covenant), "armcov");
        assert_eq!(default_file_name(EntityKind::Character), "character.armc");
        assert_eq!(default_file_name(EntityKind::Covenant), "covenant.armcov");
    }

    #[test]
    fn markdown_default_name_shares_the_base_name_with_the_save_default() {
        assert_eq!(
            default_markdown_file_name(EntityKind::Character),
            "character.md"
        );
        assert_eq!(
            default_markdown_file_name(EntityKind::Covenant),
            "covenant.md"
        );
    }

    #[test]
    fn markdown_name_follows_the_current_save_file() {
        // The save file's base name, re-extensioned: an export is named after the
        // document it came from.
        assert_eq!(
            markdown_file_name_for(Some("/home/u/gerhard.armc"), EntityKind::Character),
            "gerhard.md"
        );
        // A path recorded on Windows separates with backslashes.
        assert_eq!(
            markdown_file_name_for(Some("C:\\saves\\gerhard.armcov"), EntityKind::Covenant),
            "gerhard.md"
        );
        // A base name without an extension is already the stem.
        assert_eq!(
            markdown_file_name_for(Some("/x/gerhard"), EntityKind::Character),
            "gerhard.md"
        );
    }

    #[test]
    fn markdown_name_falls_back_to_the_kind_default() {
        // Never saved: nothing to name the export after.
        assert_eq!(
            markdown_file_name_for(None, EntityKind::Character),
            default_markdown_file_name(EntityKind::Character)
        );
        assert_eq!(
            markdown_file_name_for(None, EntityKind::Covenant),
            default_markdown_file_name(EntityKind::Covenant)
        );
        // A base name that is nothing but an extension leaves an empty stem.
        assert_eq!(
            markdown_file_name_for(Some("/x/.armc"), EntityKind::Character),
            default_markdown_file_name(EntityKind::Character)
        );
        // A directory-only path has no base name at all.
        assert_eq!(
            markdown_file_name_for(Some("/x/"), EntityKind::Covenant),
            default_markdown_file_name(EntityKind::Covenant)
        );
    }

    #[test]
    fn save_file_directory_is_the_path_up_to_its_last_separator() {
        assert_eq!(
            save_file_directory(Some("/home/u/gerhard.armc")),
            Some("/home/u/")
        );
        assert_eq!(
            save_file_directory(Some("C:\\saves\\gerhard.armcov")),
            Some("C:\\saves\\")
        );
        // A root-level file keeps the separator, so the directory is not empty.
        assert_eq!(save_file_directory(Some("/gerhard.armc")), Some("/"));
        // A bare file name names no directory; so does a document never saved.
        assert_eq!(save_file_directory(Some("gerhard.armc")), None);
        assert_eq!(save_file_directory(None), None);
    }

    #[test]
    fn ensure_extension_only_fills_when_missing() {
        // No extension -> append the kind's extension.
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        // An explicit extension the user typed is kept (incl. .armc and .json).
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.armc"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.json"), "armc"),
            PathBuf::from("/tmp/testchar.json")
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
        use arm_rules::{AbilityScore, EntityKind, RulesetRef};

        // The engine requires at least one V/F category-tagged "personality"
        // item once a catalogue is shipped at all.
        const ITEMS: &str = r#"[
          { "id": "flaw.filler", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
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
        // One node past MAX_XP_SOLVE_NODES: rejected before the solve runs, so
        // this stays fast regardless of count.
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
}
