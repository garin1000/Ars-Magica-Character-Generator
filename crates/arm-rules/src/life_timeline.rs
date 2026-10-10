//! The life timeline: a character's life laid out as chronological spans, each
//! with what its years earned and how much of that the character's purchases
//! fill. Display only — derived from the life-stage budget
//! ([`crate::LifeStageRules::budget`]) and an already-solved [`XpAllocation`],
//! never fed back into either, so the experience solve is untouched.
//!
//! The spans follow the rules' own periods: "For grogs and companions they are
//! acquired in two blocks: early childhood, and later life. For magi, there are
//! two more periods to consider: apprenticeship, and life as a magus after that."
//! (ArMDE:2432.) The player's named life sections split the open, last period —
//! later life for a grog or companion, the years after the Gauntlet for a magus.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::effective::{
    LifeStageBlock, XpAllocation, XpPoolOrigin, has_childhood_replacement, resolved_spell_level,
    restricted_pools_from_normal_budget, spell_levels_base, spell_levels_bonus,
};
use crate::life_stage::{
    LabYears, LifeStageBudget, LifeStagePlan, LifeStageRules, extra_apprenticeship_years,
    truncated_apprentice_years_completed,
};
use crate::ruleset::Ruleset;
use crate::types::{Entity, Id};

/// A character's life as chronological entries, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LifeTimeline {
    /// The spans, oldest first and contiguous: each starts at the age the one
    /// before it ends.
    pub entries: Vec<TimelineEntry>,
    /// How many years the named sections claim beyond the open span — a note,
    /// not an entry. The sections are clipped to the years actually lived.
    pub overrun_years: u32,
    /// The experience grants a Virtue or Flaw adds on top of the years (Educated,
    /// Warrior, Well-Traveled): drawn from no span, so shown beside the timeline.
    pub outside: Vec<OutsideLump>,
}

/// One span of the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Which span this is.
    pub kind: TimelineEntryKind,
    /// The age the span starts at.
    pub start_age: u32,
    /// The age the span ends at.
    pub end_age: u32,
    /// How many years the span covers (`end_age - start_age`).
    pub years: u32,
    /// What the span's years earned.
    pub capacity: EntryCapacity,
    /// Experience the character's purchases draw from this span.
    pub filled_xp: u32,
    /// Levels of spells the character's spells draw from this span.
    pub filled_spell_levels: u32,
}

/// Which span a [`TimelineEntry`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TimelineEntryKind {
    /// A span the rules fix, which the player cannot split.
    Fixed {
        /// Which one.
        section: FixedSection,
    },
    /// The player's named life section at this index of
    /// [`crate::LifeStagePlan::life_sections`].
    Named {
        /// The section's index.
        index: usize,
    },
    /// The open span's years outside every named section.
    Remaining,
}

/// The spans the rules fix. A display taxonomy of its own, deliberately not
/// [`crate::LifeStageBlock`]: that enum names funding pools, and a magus's own
/// apprenticeship and an Abandoned Apprentice's truncated training are no pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedSection {
    /// "the first five years of life" (ArMDE:2446).
    Childhood,
    /// The later-life years before an apprenticeship: a magus's, a Redcap's, or
    /// the years before an Abandoned Apprentice's training began.
    LaterLife,
    /// The fifteen years of apprenticeship (ArMDE:2503), or a Redcap's
    /// "fifteen years as an apprentice" (ArMDE:4919).
    Apprenticeship,
    /// An Abandoned Apprentice's training, cut short (ArMDE:5714).
    TruncatedTraining,
}

impl FixedSection {
    /// Every fixed span, the single source of the set (the labels are checked
    /// against it).
    pub const ALL: [FixedSection; 4] = [
        FixedSection::Childhood,
        FixedSection::LaterLife,
        FixedSection::Apprenticeship,
        FixedSection::TruncatedTraining,
    ];
}

/// The span's slug, as its serde form spells it: what `life-section-fixed-<slug>`
/// labels it by.
impl fmt::Display for FixedSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FixedSection::Childhood => "childhood",
            FixedSection::LaterLife => "later_life",
            FixedSection::Apprenticeship => "apprenticeship",
            FixedSection::TruncatedTraining => "truncated_training",
        })
    }
}

/// What a span's years earned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum EntryCapacity {
    /// Experience and levels of spells as two numbers that cannot be traded:
    /// "240 experience points, and 120 levels of spells" (ArMDE:2503).
    Separate {
        /// Experience points.
        xp: u32,
        /// Levels of spells.
        spell_levels: u32,
    },
    /// Points that are each "an experience point in an Art or Ability or one
    /// level of spell" (ArMDE:2539): a magus's years after the Gauntlet.
    Shared {
        /// The points.
        points: u32,
    },
}

/// A Virtue's or Flaw's experience grant that no span of years pays for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutsideLump {
    /// The granting item.
    pub item: Id,
    /// Experience granted.
    pub amount: u32,
    /// Experience the character's purchases draw from it.
    pub used: u32,
}

/// The character's life timeline, or `None` when there is none to draw: the life
/// sections are switched off, the character is funded from a flat pool, or it has
/// no life-stage budget.
///
/// `allocation` must be the solve of this same `entity` (from
/// [`crate::checked_xp_allocation`]): the fill is read off it, never re-solved.
///
/// The spans, oldest first:
/// - **Childhood**, filled by childhood's own pools.
/// - For anyone trained before the open span (a magus, a Redcap, an Abandoned
///   Apprentice): a fixed **later life**, then the fixed **training** block.
/// - The **open span** — later life for a grog or companion, the years after
///   the training otherwise — split into the named sections, then Remaining.
///
/// The general experience (plus every earmark carved from it) is poured oldest
/// first into the spans the general pool pays for, each capped at what its
/// years earned; levels of spells are poured after it, into the spell capacity
/// and the post-Gauntlet points the experience left (ArMDE:2539).
pub fn life_timeline(
    entity: &Entity,
    ruleset: &Ruleset,
    allocation: &XpAllocation,
) -> Option<LifeTimeline> {
    let plan = entity
        .life_stages
        .as_ref()
        .filter(|plan| plan.life_sections_enabled)?;
    let rules = ruleset.life_stages()?;
    let budget = rules.budget(entity, ruleset)?;
    let pools = PoolReading::of(entity, ruleset, allocation);
    let layout = Layout::of(entity, ruleset, rules, &budget);

    let mut drafts = vec![childhood_draft(rules, &budget, &pools)];
    drafts.extend(training_drafts(
        entity, ruleset, &layout, &budget, &pools, allocation,
    ));
    let (open_drafts, overrun_years) = open_span_drafts(plan, rules, &layout, &budget);
    drafts.extend(open_drafts);

    pour_general_xp(&mut drafts, pools.general_fill(allocation));
    pour_spell_levels(&mut drafts, spell_levels_used(entity, ruleset));

    Some(LifeTimeline {
        entries: drafts.into_iter().map(|draft| draft.entry).collect(),
        overrun_years,
        outside: pools.outside,
    })
}

// --- What the allocation says about each pool ---------------------------------

/// The solved pools, sorted by the span they belong to.
#[derive(Default)]
struct PoolReading {
    /// Childhood's pools (native language and spread, or a replacement such as
    /// Feral Upbringing), summed: `(amount, used)`.
    childhood: (u32, u32),
    /// Whether a replacement funds childhood in place of the ordinary pools.
    childhood_replaced: bool,
    /// What the later-life pool of a magus or Abandoned Apprentice funds.
    later_life_used: u32,
    /// A Redcap's apprenticeship pool: `(amount, used)`.
    apprenticeship: (u32, u32),
    /// What the earmarks carved out of the general budget fund (Church
    /// Upbringing): spent from the years, so part of the timeline's fill.
    earmarks_used: u32,
    /// The grants on top of the years (Educated, Warrior, Well-Traveled).
    outside: Vec<OutsideLump>,
}

impl PoolReading {
    fn of(entity: &Entity, ruleset: &Ruleset, allocation: &XpAllocation) -> Self {
        let earmark_flags = restricted_pools_from_normal_budget(entity, ruleset);
        let mut reading = Self {
            childhood_replaced: has_childhood_replacement(entity, ruleset),
            ..Self::default()
        };
        for (index, pool) in allocation.restricted.iter().enumerate() {
            let is_earmark = earmark_flags.get(index).copied().unwrap_or(false);
            match &pool.origin {
                XpPoolOrigin::Item { .. } if is_earmark => {
                    reading.earmarks_used = reading.earmarks_used.saturating_add(pool.used);
                }
                XpPoolOrigin::Item { item } => reading.outside.push(OutsideLump {
                    item: item.clone(),
                    amount: pool.amount,
                    used: pool.used,
                }),
                XpPoolOrigin::LifeStage { block } => reading.add_life_stage(*block, pool),
            }
        }
        reading
    }

    fn add_life_stage(&mut self, block: LifeStageBlock, pool: &crate::RestrictedXpPool) {
        match block {
            LifeStageBlock::ChildhoodNativeLanguage | LifeStageBlock::ChildhoodSpread => {
                self.childhood = (
                    self.childhood.0.saturating_add(pool.amount),
                    self.childhood.1.saturating_add(pool.used),
                );
            }
            LifeStageBlock::LaterLife => {
                self.later_life_used = self.later_life_used.saturating_add(pool.used);
            }
            LifeStageBlock::Apprenticeship => {
                self.apprenticeship = (
                    self.apprenticeship.0.saturating_add(pool.amount),
                    self.apprenticeship.1.saturating_add(pool.used),
                );
            }
        }
    }

    /// The experience the years themselves pay for: the general pool's use plus
    /// the earmarks carved out of it.
    fn general_fill(&self, allocation: &XpAllocation) -> u32 {
        allocation.general_used.saturating_add(self.earmarks_used)
    }
}

// --- Where the spans fall -----------------------------------------------------

/// The character's shape: which block of training, if any, comes before the
/// open span.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A grog or companion: later life is the open span.
    Untrained,
    /// A magus: its apprenticeship, then the years after the Gauntlet.
    Magus,
    /// A Redcap: "trained in a similar manner to magi" (ArMDE:4919).
    Redcap,
    /// An Abandoned Apprentice: training cut short (ArMDE:5714, D3/D64).
    AbandonedApprentice,
}

/// The ages the spans fall at.
struct Layout {
    shape: Shape,
    /// The age childhood ends at.
    childhood_end: u32,
    /// The age the training block starts at (= `childhood_end` when untrained).
    training_start: u32,
    /// The age the training block ends at, where the open span starts.
    training_end: u32,
    /// The age the open span ends at: the character's own, never before
    /// childhood's end.
    life_end: u32,
}

impl Layout {
    fn of(
        entity: &Entity,
        ruleset: &Ruleset,
        rules: &LifeStageRules,
        budget: &LifeStageBudget,
    ) -> Self {
        let childhood_end = rules.childhood.years;
        let life_end = entity.age.unwrap_or(0).max(childhood_end);
        let shape = shape_of(entity, ruleset, rules, budget);
        let (anchor, years) = training_anchor(shape, entity, ruleset, rules, budget);
        // Clamped like the Redcap's (plan DECIDED): a training that would end
        // after the character's age ends at it, and starts its years earlier.
        let training_end = anchor.clamp(childhood_end, life_end);
        let training_start = training_end.saturating_sub(years).max(childhood_end);
        Self {
            shape,
            childhood_end,
            training_start,
            training_end,
            life_end,
        }
    }

    fn open_span_years(&self) -> u32 {
        self.life_end - self.training_end
    }
}

fn shape_of(
    entity: &Entity,
    ruleset: &Ruleset,
    rules: &LifeStageRules,
    budget: &LifeStageBudget,
) -> Shape {
    if budget.apprenticeship_years > 0 {
        return Shape::Magus;
    }
    if is_abandoned_apprentice(entity, ruleset, rules) {
        return Shape::AbandonedApprentice;
    }
    if extra_apprenticeship_years(entity, ruleset) > 0 {
        return Shape::Redcap;
    }
    Shape::Untrained
}

/// The same test [`LifeStageRules::budget`] applies before it splits later life
/// around a truncated training.
fn is_abandoned_apprentice(entity: &Entity, ruleset: &Ruleset, rules: &LifeStageRules) -> bool {
    let Some(plan) = entity.life_stages.as_ref() else {
        return false;
    };
    rules.apprenticeship_of(entity, ruleset).is_none()
        && truncated_apprentice_years_completed(entity, ruleset).is_some()
        && rules.truncated_apprenticeship_start(plan).is_some()
}

/// Where the training block would end before any clamp, and how many years it
/// lasts.
fn training_anchor(
    shape: Shape,
    entity: &Entity,
    ruleset: &Ruleset,
    rules: &LifeStageRules,
    budget: &LifeStageBudget,
) -> (u32, u32) {
    let childhood_end = rules.childhood.years;
    match shape {
        Shape::Untrained => (childhood_end, 0),
        Shape::Magus => (budget.gauntlet_age, budget.apprenticeship_years),
        // Plan DECIDED: the fifteen years end at the ruleset's default Gauntlet
        // age, since a Redcap's budget names no Gauntlet of its own.
        Shape::Redcap => {
            let age = entity.age.unwrap_or(0);
            let anchor = rules
                .apprenticeship
                .as_ref()
                .and_then(|block| block.default_gauntlet_age)
                .unwrap_or(age);
            (anchor.min(age), extra_apprenticeship_years(entity, ruleset))
        }
        // D64: the pre-span ends where the training starts.
        Shape::AbandonedApprentice => (
            childhood_end
                .saturating_add(budget.later_life_years)
                .saturating_add(budget.truncated_training_years),
            budget.truncated_training_years,
        ),
    }
}

// --- The spans ----------------------------------------------------------------

/// An entry under construction, with whether the general pool pays for it.
struct Draft {
    entry: TimelineEntry,
    fed_by_general: bool,
}

impl Draft {
    fn new(kind: TimelineEntryKind, start_age: u32, end_age: u32, capacity: EntryCapacity) -> Self {
        Self {
            entry: TimelineEntry {
                kind,
                start_age,
                end_age,
                years: end_age - start_age,
                capacity,
                filled_xp: 0,
                filled_spell_levels: 0,
            },
            fed_by_general: true,
        }
    }

    /// A span its own pool pays for, filled with that pool's use.
    fn filled_by_own_pool(mut self, used: u32) -> Self {
        self.entry.filled_xp = used;
        self.fed_by_general = false;
        self
    }
}

fn xp_only(xp: u32) -> EntryCapacity {
    EntryCapacity::Separate {
        xp,
        spell_levels: 0,
    }
}

fn fixed(section: FixedSection) -> TimelineEntryKind {
    TimelineEntryKind::Fixed { section }
}

/// Childhood: the budget's 75 + 45 (ArMDE:2446), which the years earn whether
/// or not a native language has been picked (without one the solve builds no
/// native-language pool) — unless a replacement such as Feral Upbringing funds
/// childhood instead, when the span shows that pool's own amount (ArMDE:6183).
fn childhood_draft(rules: &LifeStageRules, budget: &LifeStageBudget, pools: &PoolReading) -> Draft {
    let (pooled, used) = pools.childhood;
    let amount = if pools.childhood_replaced {
        pooled
    } else {
        budget
            .childhood_native_xp
            .saturating_add(budget.childhood_spread_xp)
    };
    Draft::new(
        fixed(FixedSection::Childhood),
        0,
        rules.childhood.years,
        xp_only(amount),
    )
    .filled_by_own_pool(used)
}

/// The fixed later life and the training block, for a trained shape; nothing
/// for a grog or companion.
fn training_drafts(
    entity: &Entity,
    ruleset: &Ruleset,
    layout: &Layout,
    budget: &LifeStageBudget,
    pools: &PoolReading,
    allocation: &XpAllocation,
) -> Vec<Draft> {
    let later_life = |capacity: u32| {
        Draft::new(
            fixed(FixedSection::LaterLife),
            layout.childhood_end,
            layout.training_start,
            xp_only(capacity),
        )
    };
    let training = |section: FixedSection, capacity: EntryCapacity| {
        Draft::new(
            fixed(section),
            layout.training_start,
            layout.training_end,
            capacity,
        )
    };
    let spell_base = spell_levels_base(entity, ruleset.profile(&entity.type_id));
    let spell_bonus = spell_levels_bonus(entity, ruleset);
    match layout.shape {
        Shape::Untrained => Vec::new(),
        // Later life "until apprenticeship for magi" (ArMDE:2282) is its own
        // pool; the apprenticeship's experience and levels (ArMDE:2503) take
        // Skilled/Weak Parens (ArMDE:5037).
        Shape::Magus => vec![
            later_life(budget.later_life_xp).filled_by_own_pool(pools.later_life_used),
            training(
                FixedSection::Apprenticeship,
                EntryCapacity::Separate {
                    xp: add_signed(budget.apprenticeship_xp, allocation.general_bonus),
                    spell_levels: add_signed(spell_base, spell_bonus),
                },
            ),
        ],
        // A Redcap's later life is general; its fifteen years are its own pool.
        Shape::Redcap => {
            let years = layout.training_start - layout.childhood_end;
            let (amount, used) = pools.apprenticeship;
            vec![
                later_life(years.saturating_mul(budget.later_life_rate)),
                training(FixedSection::Apprenticeship, xp_only(amount)).filled_by_own_pool(used),
            ]
        }
        Shape::AbandonedApprentice => vec![
            later_life(budget.later_life_xp).filled_by_own_pool(pools.later_life_used),
            training(
                FixedSection::TruncatedTraining,
                EntryCapacity::Separate {
                    xp: add_signed(budget.truncated_training_xp, allocation.general_bonus),
                    spell_levels: add_signed(
                        budget
                            .truncated_training_spell_levels
                            .saturating_add(spell_base),
                        spell_bonus,
                    ),
                },
            ),
        ],
    }
}

/// The open span: the named sections, clipped to the years it holds, then
/// Remaining. Returns the entries and the years the sections claim beyond it.
fn open_span_drafts(
    plan: &LifeStagePlan,
    rules: &LifeStageRules,
    layout: &Layout,
    budget: &LifeStageBudget,
) -> (Vec<Draft>, u32) {
    let capacity = |years: u32, lab_years: LabYears| match layout.shape {
        // "For every year, the magus gets 30 points" (ArMDE:2539), less each
        // section's own lab seasons (ArMDE:2550).
        Shape::Magus => EntryCapacity::Shared {
            points: rules
                .post_apprenticeship
                .as_ref()
                .map_or(0, |post| post.points(years, lab_years)),
        },
        _ => xp_only(years.saturating_mul(budget.later_life_rate)),
    };
    let mut drafts = Vec::with_capacity(plan.life_sections.len().saturating_add(1));
    let mut start = layout.training_end;
    let mut years_left = layout.open_span_years();
    let mut overrun_years = 0u32;
    for (index, section) in plan.life_sections.iter().enumerate() {
        let lived = section.years.min(years_left);
        years_left -= lived;
        overrun_years = overrun_years.saturating_add(section.years - lived);
        drafts.push(Draft::new(
            TimelineEntryKind::Named { index },
            start,
            start + lived,
            capacity(lived, section.lab_years),
        ));
        start += lived;
    }
    drafts.push(Draft::new(
        TimelineEntryKind::Remaining,
        start,
        start + years_left,
        capacity(years_left, plan.post_gauntlet_lab_years),
    ));
    (drafts, overrun_years)
}

// --- Pouring ------------------------------------------------------------------

/// Pours the years' experience into the spans the general pool pays for, oldest
/// first, each up to what it earned.
fn pour_general_xp(drafts: &mut [Draft], mut xp: u32) {
    for draft in drafts.iter_mut().filter(|draft| draft.fed_by_general) {
        let room = match draft.entry.capacity {
            EntryCapacity::Separate { xp, .. } => xp,
            EntryCapacity::Shared { points } => points,
        };
        draft.entry.filled_xp = take(&mut xp, room);
    }
}

/// Pours the levels of spells, oldest first, into each span's spell capacity —
/// or, for post-Gauntlet points, into what the experience left of them
/// (ArMDE:2539).
fn pour_spell_levels(drafts: &mut [Draft], mut levels: u32) {
    for draft in drafts.iter_mut() {
        let room = match draft.entry.capacity {
            EntryCapacity::Separate { spell_levels, .. } => spell_levels,
            EntryCapacity::Shared { points } => points.saturating_sub(draft.entry.filled_xp),
        };
        draft.entry.filled_spell_levels = take(&mut levels, room);
    }
}

/// Takes up to `room` out of `supply`.
fn take(supply: &mut u32, room: u32) -> u32 {
    let taken = (*supply).min(room);
    *supply -= taken;
    taken
}

/// The levels the character's spells consume, summed saturating so a crafted
/// spell list cannot overflow.
fn spell_levels_used(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .spells
        .iter()
        .filter_map(|spell| resolved_spell_level(spell, ruleset))
        .fold(0u32, u32::saturating_add)
}

/// `base` moved by a signed bonus (Skilled Parens +60, Weak Parens -60),
/// floored at 0.
fn add_signed(base: u32, delta: i64) -> u32 {
    let moved = i64::from(base).saturating_add(delta).max(0);
    u32::try_from(moved).unwrap_or(u32::MAX)
}
