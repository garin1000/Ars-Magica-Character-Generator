//! Aging: the yearly roll every character makes once age catches up with them,
//! and the two tables that decide what it costs.
//!
//! "Characters begin aging in the Winter after they turn 35. Every year, a
//! character must roll on the aging table." — **AGING TOTAL: Stress die (no
//! botch) + age/10 (round up) - Living Conditions modifier - Longevity Ritual
//! modifier**
//! (Source: Ars Magica - Definitive Edition (Core Rules).md:16563-16617.)
//!
//! # The engine never rolls
//!
//! `arm-rules` has no `rand` dependency and never will. The stress die is thrown
//! at the table by the player, who types the result in; everything here is the
//! arithmetic *around* that number — the age term, the two modifiers, and which
//! row of the table the total lands on. A generator that rolled for the player
//! would be inventing rules-relevant state no one at the table agreed to, and it
//! would make a character's history unreproducible from its save file.
//!
//! # Not here
//!
//! The same line is drawn around the **Crisis** an aging row can send a
//! character to ([`AgingRowEffect::NextDecrepitudeLevelAndCrisis`],
//! `:16619-16638`). The engine does the arithmetic and the look-up: the CRISIS
//! TOTAL of `:16621`, which row of the Crisis Table (`:16624-16632`) that total
//! lands on, the Ease Factor of the Stamina roll it calls for, and the level of
//! the Creo Corpus Ritual that would resolve it (`:16638`).
//!
//! What it never does is **resolve survival**. It does not throw the Stamina
//! die, does not pronounce a character survived or died, and never kills one.
//! Those are the table's to decide and the player's to record — the engine only
//! lays out what the roll is against.
//!
//! See `RULES.md` for the provenance of every value the shipped
//! `rules/core/aging.json` carries.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::characteristics::Characteristic;
use crate::effective::{decrepitude_points_total, decrepitude_score, selections_for_effects};
use crate::ruleset::Ruleset;
use crate::types::{AgingEffect, AgingLogEntry, Effect, Entity, Id, SourceRef, is_false};

/// The aging rules, loaded from `rules/core/aging.json`.
// No `Default`: every field is authored data with no meaningful zero (an aging
// table with no rows is not a default, it is a broken file), and the ruleset
// holds these as an `Option` for the absent case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingRules {
    /// The age after which a character rolls every year: "Characters begin aging
    /// in the Winter after they turn 35" (`:16565`).
    pub start_age: u32,
    /// What the age term of the aging total is divided by, rounding up:
    /// "age/10 (round up)" (`:16567`).
    pub age_divisor: u32,
    /// The lowest total at which "the character's apparent age increases by one
    /// year" (`:16600`); below it, "No apparent aging" (`:16599`). Signed
    /// because the two modifiers are subtracted from the total, which can
    /// therefore land below zero.
    pub apparent_age_increase_min: i32,
    /// The clamp a Longevity Ritual puts on the roll before [`Self::start_age`]
    /// (`:16575`), when the ruleset ships one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longevity_clamp: Option<LongevityClamp>,
    /// The Living Conditions table (`:16581-16594`), in file order.
    #[serde(default)]
    pub living_conditions: Vec<LivingCondition>,
    /// The Aging Roll table (`:16597-16611`), in file order.
    #[serde(default)]
    pub outcomes: Vec<AgingRow>,
    /// The Crisis Table and its two rolls (`:16619-16634`), when the ruleset
    /// ships them. Optional so an aging block written before the crisis existed
    /// keeps loading unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crisis: Option<CrisisRules>,
    /// The Decrepitude score at which a character is "extremely frail, and must
    /// roll on the Crisis Table if they undertake stressful activities, such as
    /// long journeys, or any combat" (`:16617`) — 4 in the core rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frail_decrepitude_score: Option<u8>,
    /// The Decrepitude score at which a character is "bedridden and will die
    /// within a few months at most. They cannot be saved by mortal intervention."
    /// (`:16617`) — 5 in the core rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fatal_decrepitude_score: Option<u8>,
}

impl AgingRules {
    /// The first age at which a roll is owed: [`Self::start_age`] + 1, the one
    /// deliberate off-by-one in the engine.
    ///
    /// "Characters begin aging in the Winter **after** they turn 35. Every year, a
    /// character must roll on the aging table." (`:16565`) The Winter after the
    /// 35th birthday falls in the character's 36th year, so 35 is the last age
    /// owing nothing and 36 is the first owing a roll. The creation-time rule says
    /// the same thing from the other side: "a character **over** the age of 35
    /// must make aging rolls … before the game begins" (`:2232`).
    ///
    /// `:2496` reads against this and is **disposed of, not ignored**: "you should
    /// also make aging rolls for the character each year **from the age of 35**".
    /// That sentence is advice inside the worked magus-advancement example, while
    /// `:2232` is the creation-time rule the app enforces — and `:2232` agrees
    /// with `:16565`. So 36 it is.
    ///
    /// No month or season is modelled. "The Winter after they turn 35" places the
    /// roll *inside* a year rather than splitting one, and the app tracks no
    /// seasons, so a year either owes a roll or it does not.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16565, :2232,
    /// :2496.
    pub fn first_roll_age(&self) -> u32 {
        self.start_age.saturating_add(1)
    }

    /// The age term of the AGING TOTAL: "age/10 (round up)" (`:16567`), with
    /// [`Self::age_divisor`] standing in for the 10.
    ///
    /// Rounding up means the term steps on the first year of each decade, not the
    /// last: 30 still scores 3, 31 already scores 4. Returns 0 for a divisor of 0,
    /// which [`Ruleset::validate_integrity`] rejects at load — a ruleset that
    /// reached the engine anyway has no age term to compute, not a panic to raise.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16567.
    pub fn age_modifier(&self, age: u32) -> i32 {
        if self.age_divisor == 0 {
            return 0;
        }
        i32::try_from(age.div_ceil(self.age_divisor)).unwrap_or(i32::MAX)
    }
}

/// One year on a character's aging schedule: the `age` he reaches and, when the
/// character has a birth year, the calendar `year` that age falls in.
///
/// Both, because they answer different questions.
/// [`AgingRules::age_modifier`] wants the age, while
/// [`AgingLogEntry`](crate::types::AgingLogEntry) records a **calendar** year —
/// so pairing them is the schedule's job rather than every caller's.
///
/// [`Self::recorded`] answers the third question — whether this year's roll has
/// already been made — by matching the log on [`AgingLogEntry::age`] rather than
/// on the calendar year, which is unavailable without a birth year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingYear {
    /// The age the character reaches in this year of the schedule.
    pub age: u32,
    /// The calendar year that age falls in — `birth_year + age` — or `None` when
    /// the character has no birth year recorded.
    pub year: Option<i32>,
    /// Whether [`resolve_year`] has already resolved this year — i.e. whether
    /// the log carries an entry for this `age`. A second apply of a recorded
    /// year is refused ([`AgingError::YearAlreadyRecorded`]), so this is what
    /// lets a UI offer the years still owed.
    pub recorded: bool,
}

/// The calendar year a character's `age` falls in — `birth_year + age` — or
/// `None` when no birth year is recorded.
///
/// Shared by the schedule and the writer so a scheduled year and the log entry
/// that resolves it can never disagree about which calendar year it was.
fn calendar_year(entity: &Entity, age: u32) -> Option<i32> {
    entity
        .birth_year
        .zip(i32::try_from(age).ok())
        .map(|(birth, elapsed)| birth.saturating_add(elapsed))
}

/// Whether the log already carries a resolved entry for `age`.
///
/// The age is the key, not the calendar year: a character with no birth year has
/// no calendar year to be addressed by, and a hand-written free-text entry
/// carries no age at all and so is never matched here.
fn logged_year(entity: &Entity, age: u32) -> Option<&AgingLogEntry> {
    entity.aging_log.iter().find(|entry| entry.age == Some(age))
}

/// Every aging roll a character owes, ascending, from
/// [`AgingRules::first_roll_age`] through his current age.
///
/// Empty when the question does not arise: the character is at or under the
/// threshold, his age was never entered, or the ruleset ships no aging rules at
/// all (which stands the whole subsystem down rather than letting the engine
/// invent a threshold).
///
/// **A Longevity Ritual holder under 35 is not scheduled**, which is a decision
/// rather than an oversight. `:16575` says such a character "should roll on the
/// table no matter what his age" — but that clause is unbounded downward, nothing
/// on the entity records *when* the ritual was made, and those rolls are clamped
/// (see [`LongevityClamp`]) so they can never grant an Aging Point. The obligation
/// the app enforces is `:2232`'s, which is age-gated and carries no ritual clause.
/// So the schedule yields `first_roll_age()..=age` and nothing below it; a later
/// step's aging total will still compute a pre-35 roll correctly for a caller that
/// asks for one.
///
/// **Known gap — a trait may move the start age, and none does here.** Strong
/// Faerie Blood reads "You start making aging rolls at the age of **fifty**,
/// rather than the normal 35, and get -3 to Aging Rolls" (`:5036`). The -3 is
/// implemented (as an `aging_roll` modifier on the shipped item, folded in by
/// [`aging_total`]); the start-at-fifty half is **not**, so such a character is
/// still scheduled from [`AgingRules::first_roll_age`]. It would need a per-trait
/// override of [`AgingRules::start_age`], which no other shipped item asks for.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16565, :16575, :2232,
/// :5036.
pub fn aging_schedule(entity: &Entity, ruleset: &Ruleset) -> Vec<AgingYear> {
    let (Some(rules), Some(age)) = (ruleset.aging(), entity.age) else {
        return Vec::new();
    };
    (rules.first_roll_age()..=age)
        .map(|age| AgingYear {
            age,
            year: calendar_year(entity, age),
            recorded: logged_year(entity, age).is_some(),
        })
        .collect()
}

/// The resolved Living Conditions modifier — the number the AGING TOTAL
/// subtracts (`:16567-16569`), broken into the two places it comes from.
///
/// Split rather than a bare integer because the sheet has to *show* the
/// arithmetic: which rows the character lives under, and how much of the figure
/// is his Virtues and Flaws rather than his circumstances.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16594.
// `Serialize` only: this is a computed read-out that crosses the IPC edge for the
// sheet to show, never something a save carries back in.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct LivingConditionsModifier {
    /// The chosen rows that resolved against the table, in canonical id order.
    pub rows: Vec<Id>,
    /// Σ of those rows' [`LivingCondition::modifier`].
    pub from_table: i32,
    /// Σ of the Virtue/Flaw `living_conditions` aging modifiers.
    pub from_traits: i32,
    /// `from_table + from_traits` — the modifier the AGING TOTAL subtracts.
    pub total: i32,
}

/// Resolves a character's [`Entity::living_conditions`] against the Living
/// Conditions table, and adds the Virtue/Flaw modifiers that name the same
/// subsystem.
///
/// # The sign is the book's, not the formula's
///
/// "A high Longevity Ritual modifier and a high Living Conditions modifier both
/// indicate longer life" (`:16571`) — which works because the AGING TOTAL
/// *subtracts* the modifier. So the numbers here are the ones the table and the
/// descriptors print (Wealthy +2, Leper -2, Mild Aging +1, Poor Living Conditions
/// -1), and the negation happens once, later, in the total. Returning the
/// negated figure from here would double-negate it there.
///
/// # An unknown id contributes nothing
///
/// An id the table does not carry is skipped silently: the engine has **one
/// evaluation path**, so a computation never refuses to produce a number — the
/// unresolved id is reported as a validation finding instead. Trait modifiers are
/// summed even when the ruleset ships no aging table at all; a Virtue's bonus does
/// not depend on a table being present to be worth what it says.
///
/// Sources: Ars Magica - Definitive Edition (Core Rules).md:16567-16571 (the
/// total and the sign), `:16581-16594` (the table), `:4530` (Mild Aging +1),
/// `:6340` (Leprosy -2), `:6620` (Poor Living Conditions -1).
pub fn living_conditions_modifier(entity: &Entity, ruleset: &Ruleset) -> LivingConditionsModifier {
    let mut rows = Vec::new();
    let mut from_table = 0;
    if let Some(table) = ruleset.aging().map(|rules| &rules.living_conditions) {
        // Walking the entity's set (a `BTreeSet`, so canonically ordered) rather
        // than the table keeps `rows` in id order whatever order the file lists.
        for id in &entity.living_conditions {
            let Some(row) = table.iter().find(|row| row.id == *id) else {
                continue;
            };
            rows.push(id.clone());
            from_table += i32::from(row.modifier);
        }
    }

    let mut from_traits = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::AgingMod {
                kind: AgingEffect::LivingConditions,
                amount,
            } = effect
            {
                from_traits += i32::from(*amount);
            }
        }
    }

    LivingConditionsModifier {
        rows,
        from_table,
        from_traits,
        total: from_table + from_traits,
    }
}

/// One year's AGING TOTAL, broken into every term that made it.
///
/// Split rather than a bare number because the sheet has to *show* the
/// arithmetic — a player who cannot see which term moved the total cannot check
/// it against the book.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569.
// `Serialize` only, like every other aging read-out: the aging calculator shows
// these terms, and none of them is ever read back off a save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgingTotal {
    /// The actual age the roll was made at (`:16577`).
    pub age: u32,
    /// The player's stress die, as typed in.
    pub die: i32,
    /// ⌈`age` / [`AgingRules::age_divisor`]⌉ — ADDED.
    pub age_modifier: i32,
    /// The Living Conditions Modifier — SUBTRACTED (`:16571`).
    pub living_conditions: LivingConditionsModifier,
    /// The Longevity Ritual bonus — SUBTRACTED. 0 with no ritual, or with a
    /// ritual whose bonus the player has not entered yet.
    pub longevity_bonus: i32,
    /// Σ of the Virtue/Flaw aging-ROLL modifiers — ADDED with their stored sign.
    pub trait_modifier: i32,
    /// The total before the `:16575` clamp.
    pub uncapped_total: i32,
    /// The total after it — the number the Aging Roll table is indexed by.
    pub total: i32,
    /// Whether the `:16575` clamp fired on this roll.
    pub capped_by_longevity: bool,
}

/// The AGING TOTAL for one year's roll (`:16567-16569`).
///
/// > **AGING TOTAL: Stress die (no botch) + age/10 (round up)**
/// > **\- Living Conditions modifier**
/// > **\- Longevity Ritual modifier**
///
/// `die` is the player's stress die (no botch), typed in — the engine never rolls
/// and `arm-rules` has no `rand` dependency. `age` is a parameter rather than read
/// off the entity, because `:2232`'s pre-play catch-up walks each owed year and
/// every year's roll uses THAT year's age. It is the ACTUAL age: "The modifier to
/// rolls depends on the character's actual, not apparent, age" (`:16577`).
///
/// # Signs
///
/// The two modifiers keep the book's own sign wherever they are stored, and are
/// negated exactly once — here. So Mild Aging's `+1` Living Conditions modifier
/// (`:4530`) *lowers* the total and Poor Living Conditions' `-1` (`:6620`) *raises*
/// it, which is what "a high … Living Conditions modifier … indicate[s] longer
/// life" (`:16571`) means. The Virtue/Flaw aging-ROLL modifiers are a different
/// quantity and are ADDED with their stored sign: Faerie Blood's `-1` (`:3801`)
/// lowers the total directly.
///
/// # The `:16575` clamp applies to the TOTAL, not to the die
///
/// > A character under the influence of a Longevity Ritual should roll on the
/// > table no matter what his age, but treats all rolls of 10 or more as rolls of
/// > 9 until he reaches the age of 35 … he is at no risk of actually aging before
/// > any other characters.
///
/// The argument, because this is easy to get backwards and at least one project
/// note has:
///
/// 1. The formula block those rolls feed is headed "AGING TOTAL", and the table's
///    index column is headed "Aging Roll" (`:16597`) — the clamp's "rolls" and the
///    table's "roll" are the same quantity.
/// 2. 10 is meaningful only in the table's index space: it is exactly where the
///    first aging-point row begins (`:16601`). On a stress die 10 is nothing at
///    all.
/// 3. Decisive: a 34-year-old average peasant with the weakest legal ritual (+1 —
///    "+1 bonus for every five points **or fraction** of Creo Corpus Lab Total",
///    `:10662`) would, under the die reading, get `9 + ⌈34/10⌉ - 1 = 12` and take
///    an Aging Point — while the same character with **no** ritual makes no roll
///    at all before 36. The die reading would make a Longevity Ritual strictly
///    worse than nothing in exactly the case the sentence calls safe.
///
/// It is a ceiling, never a floor: an uncapped total of 2 stays 2.
///
/// **The one-year seam is the text's, not a bug.** [`AgingRules::start_age`] (35,
/// `:16565`) and [`LongevityClamp::until_age`] (35, `:16575`) are two numbers from
/// two different sentences that happen to coincide. "Until he reaches the age of
/// 35" stops the clamp *at* 35, while rolls are owed only from 36 (`:16565`) — so
/// at exactly 35 a ritual-holder rolls unclamped while a character without one
/// does not roll at all. That is what the two sentences say when read together.
///
/// `None` when the ruleset ships no aging rules.
pub fn aging_total(entity: &Entity, ruleset: &Ruleset, age: u32, die: i32) -> Option<AgingTotal> {
    let rules = ruleset.aging()?;
    let age_modifier = rules.age_modifier(age);
    let living_conditions = living_conditions_modifier(entity, ruleset);

    // The one existing reader of the stored ritual bonus, reused so the aging
    // total and the Longevity read-out can never disagree. It is `None` for
    // exactly the character who has no ritual, which is also the gate a
    // `longevity_bonus` modifier needs.
    let ritual = crate::derived::longevity_bonus(entity, ruleset);
    let mut longevity_bonus = ritual.as_ref().map_or(0, |read_out| read_out.bonus);
    let mut trait_modifier = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            let Effect::AgingMod { kind, amount } = effect else {
                continue;
            };
            match kind {
                AgingEffect::AgingRoll => trait_modifier += i32::from(*amount),
                // A modifier to a ritual bonus is meaningless without a ritual.
                AgingEffect::LongevityBonus => {
                    if ritual.is_some() {
                        longevity_bonus += i32::from(*amount);
                    }
                }
                // Already summed into `living_conditions` above.
                AgingEffect::LivingConditions => {}
                // None of these is a term of the total: `no_aging` decides
                // whether Aging Points reach the Characteristics at all,
                // `no_apparent_aging` whether the appearance follows the roll
                // ([`resolve_outcome`]), and `decrepitude` modifies the accrued
                // score rather than the roll.
                //
                // The two crisis kinds are here for a stronger reason: neither
                // reaches the aging roll at all. "Virtues that affect aging rolls
                // do not affect crisis survival rolls" (`:16636`) separates the two
                // rolls, and this arm is its converse — a bonus granted
                // specifically to the survival roll (Mild Aging's +3, `:4530`) is
                // no more an aging-roll modifier than an aging-roll modifier is a
                // survival one. `crisis_heavy_wound` is not a number anywhere.
                //
                // Source: Ars Magica - Definitive Edition (Core Rules).md:4530,
                // :16636.
                AgingEffect::NoAging
                | AgingEffect::NoApparentAging
                | AgingEffect::Decrepitude
                | AgingEffect::CrisisSurvival
                | AgingEffect::CrisisHeavyWound => {}
            }
        }
    }

    let uncapped_total =
        die + age_modifier - living_conditions.total - longevity_bonus + trait_modifier;

    // "treats all rolls of 10 or more as rolls of 9 until he reaches the age of
    // 35" (`:16575`) — a ceiling on the total, applied only to a ritual-holder
    // below the clamp's age. `min` alone would silently claim a cap on a total it
    // never touched, so the flag compares.
    let mut total = uncapped_total;
    if let Some(clamp) = &rules.longevity_clamp
        && ritual.is_some()
        && age < clamp.until_age
    {
        total = uncapped_total.min(clamp.max_total);
    }

    Some(AgingTotal {
        age,
        die,
        age_modifier,
        living_conditions,
        longevity_bonus,
        trait_modifier,
        uncapped_total,
        total,
        capped_by_longevity: total < uncapped_total,
    })
}

/// What the Aging Roll table does at one total — the *reading*, and nothing
/// else. **Nothing here is written to the character**; applying an outcome (and
/// logging the year) belongs to the single writer a later step introduces.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16599-16615.
// `Serialize` only: a reading, never a stored value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgingOutcome {
    /// The total this resolves — post-clamp, so a caller can echo it back.
    pub total: i32,
    /// "Otherwise, the character's apparent age increases by one year"
    /// (`:16577`, `:16600`) — unless the character is exempt from that sentence
    /// altogether, which an [`AgingEffect::NoApparentAging`] item
    /// (`:3488`, `:5189`) makes him at every total.
    pub apparent_age_increases: bool,
    /// The Aging Points the row awards, in the order the row names them. Empty
    /// below the table's first row, which costs nothing.
    pub awards: Vec<AgingPointAward>,
    /// "… and Crisis" (`:16602`, `:16611`). Resolving the Crisis itself
    /// (`:16619-16632`) is its own slice; this only says one follows.
    pub crisis: bool,
}

/// One award an Aging Roll row makes: where the points go, and how many.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16601-16615.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgingPointAward {
    /// Which Characteristic (or which question to ask the player) the points
    /// land on.
    pub target: AgingPointTarget,
    /// How many points. `None` **only** for
    /// [`AgingPointTarget::NextDecrepitudeLevel`] when the advancement curve
    /// cannot price the next Decrepitude score — reported as unpriceable rather
    /// than silently costed at 0.
    pub points: Option<u32>,
}

/// Where an Aging Roll row's points go.
///
/// Three variants rather than two, because the UI has three genuinely different
/// questions to ask — and an exhaustive `match` makes a new kind of row a
/// compile error until every reader has decided what to do with it.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16601-16615.
// **Adjacently** tagged, so the UI can switch on `kind` and a named
// Characteristic still rides along in its own key. Internal tagging cannot carry
// a newtype variant whose content is a plain string, and an untagged enum would
// leave the UI guessing which of the three questions to ask.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "characteristic", rename_all = "snake_case")]
pub enum AgingPointTarget {
    /// The table names the Characteristic itself: rows 14-21 (`:16603-16610`).
    Named(Characteristic),
    /// "1 Aging Point in any Characteristic" (`:16601`) — "the player may choose
    /// the Characteristic" (`:16615`).
    PlayerChoice,
    /// "sufficient Aging Points (in any Characteristics) to reach the next level
    /// in Decrepitude" (`:16602`, `:16611`). The player still picks *where* the
    /// points land, and may spread them, but the COUNT is derived — see
    /// [`points_to_next_decrepitude_level`].
    NextDecrepitudeLevel,
}

/// What the aging table does at `total` (`:16599-16615`).
///
/// # Two questions, not two rows
///
/// "2 or less — No apparent aging" and "3 or more — Apparent age increases by one
/// year" (`:16599-16600`) are **not** alternatives to the effect rows: they are
/// one threshold ([`AgingRules::apparent_age_increase_min`]) asked of every
/// total, so a 20 both ages the appearance and costs two Characteristics a point.
///
/// # Why it needs the character
///
/// Rows 13 and 22+ ask for "sufficient Aging Points … to reach the next level in
/// Decrepitude" (`:16602`, `:16611`) — a count the table does not print, measured
/// off the character's accrued points and the Ability advancement curve.
///
/// # Pure
///
/// Nothing is written: not the Aging Points, not the apparent age, not the log.
/// A row that carries a Crisis sets [`AgingOutcome::crisis`] and stops there —
/// reading the Crisis Table (`:16619-16632`) needs the Decrepitude the row's own
/// award has yet to raise (`:16619`), so it belongs to [`resolve_year`], which
/// makes that award.
///
/// `None` when the ruleset ships no aging rules.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16599-16617.
pub fn resolve_outcome(entity: &Entity, ruleset: &Ruleset, total: i32) -> Option<AgingOutcome> {
    let rules = ruleset.aging()?;
    let row = rules.outcomes.iter().find(|row| row.covers(total));

    // A total below the table's first row costs nothing at all — which is a
    // result, not a missing one, so it is `Some` with no awards.
    let (awards, crisis) = match row.map(|row| &row.effect) {
        None => (Vec::new(), false),
        Some(AgingRowEffect::AnyCharacteristic { points }) => (
            vec![AgingPointAward {
                target: AgingPointTarget::PlayerChoice,
                points: Some(*points),
            }],
            false,
        ),
        // "1 Aging Point in Str and Sta" (`:16607`) gives EACH named
        // Characteristic a point, so one award per name.
        Some(AgingRowEffect::NamedCharacteristics {
            points,
            characteristics,
        }) => (
            characteristics
                .iter()
                .map(|characteristic| AgingPointAward {
                    target: AgingPointTarget::Named(*characteristic),
                    points: Some(*points),
                })
                .collect(),
            false,
        ),
        Some(AgingRowEffect::NextDecrepitudeLevelAndCrisis) => (
            vec![AgingPointAward {
                target: AgingPointTarget::NextDecrepitudeLevel,
                points: points_to_next_decrepitude_level(entity, ruleset),
            }],
            true,
        ),
    };

    Some(AgingOutcome {
        total,
        apparent_age_increases: total >= rules.apparent_age_increase_min
            && !suppresses_apparent_aging(entity, ruleset),
        awards,
        crisis,
    })
}

/// Whether the character's apparent age is exempt from `:16577` — i.e. whether
/// he carries an [`AgingEffect::NoApparentAging`] item.
///
/// It gates the appearance **only**. A Bee King "do[es] not appear to age"
/// (`:3488`) and takes the row's Aging Points like anyone else; Unaging "may
/// choose [its] apparent age freely" (`:5189`) and additionally carries
/// [`AgingEffect::NoAging`], which is a different exemption applied elsewhere.
/// Bound to (Role) carries neither this tag nor its effect: "the character's
/// apparent age advances in line with their physical age" (`:5743`).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3488, :5189, :5743.
fn suppresses_apparent_aging(entity: &Entity, ruleset: &Ruleset) -> bool {
    selections_for_effects(entity, ruleset)
        .iter()
        .filter_map(|selection| ruleset.point_items.get(&selection.item_ref))
        .flat_map(|item| &item.effects)
        .any(|effect| {
            matches!(
                effect,
                Effect::AgingMod {
                    kind: AgingEffect::NoApparentAging,
                    ..
                }
            )
        })
}

/// How many Aging Points "reach the next level in Decrepitude" (`:16602`,
/// `:16611`) costs this character.
///
/// Decrepitude is no separate curve: "Every Aging Point also counts as an
/// experience point towards Decrepitude, which increases as an Ability"
/// (`:16617`). So the count is the advancement curve's price for the score above
/// the character's current one, less the points he has already accrued — reusing
/// [`decrepitude_points_total`] and [`decrepitude_score`] rather than
/// re-deriving either.
///
/// Floored at 1: a character sitting exactly on a level boundary must still gain
/// *something*, because reaching the next level cannot cost nothing.
///
/// `None` when the curve cannot price the next score — the table tops out
/// (`AdvancementTable::max_score`), and a level with no price is reported as
/// unpriceable rather than silently costed at 0.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16602, :16611,
/// :16617.
fn points_to_next_decrepitude_level(entity: &Entity, ruleset: &Ruleset) -> Option<u32> {
    let accrued = decrepitude_points_total(entity);
    let next_score = decrepitude_score(entity, ruleset).checked_add(1)?;
    let priced = ruleset.advancement().xp_for_score(next_score)?;
    Some(priced.saturating_sub(accrued).max(1))
}

/// One Crisis's CRISIS TOTAL, broken into every term that made it.
///
/// Split rather than a bare number for the same reason [`AgingTotal`] is: the
/// sheet has to *show* the arithmetic, and a player who cannot see which term
/// moved the total cannot check it against the book. Three terms is all there
/// is (`:16621`), so all three are here.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16621.
// `Serialize` only, like every other aging read-out: the crisis calculator shows
// these terms, and none of them is ever read back off a save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrisisTotal {
    /// The age the Crisis fell in — the year's own age, not the character's
    /// current one, for the same reason [`AgingTotal::age`] is.
    pub age: u32,
    /// The player's Simple Die, as typed in.
    pub die: i32,
    /// ⌈`age` / [`AgingRules::age_divisor`]⌉ — ADDED. The aging roll's own age
    /// term ([`AgingRules::age_modifier`]), not a second reading of it.
    pub age_modifier: i32,
    /// The Decrepitude Score **as of `age`** — ADDED. See [`crisis_total`] for
    /// why "as of" is the load-bearing word.
    pub decrepitude_score: u8,
    /// `die + age_modifier + decrepitude_score` — the number the Crisis Table is
    /// indexed by.
    pub total: i32,
}

/// The CRISIS TOTAL for one Crisis (`:16621`).
///
/// > **CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**
///
/// `die` is the player's Simple Die, typed in — the engine never rolls and
/// `arm-rules` has no `rand` dependency. `age` is a parameter rather than read
/// off the entity, because the Crisis belongs to the year that flagged it and a
/// pre-play catch-up may resolve it long after later years have been applied.
///
/// # Three terms, and no fourth
///
/// `:16621` names exactly three, all ADDED, and **no trait modifier of any kind
/// reaches this total**. Nothing in any source gives a Virtue or Flaw a bearing
/// on the crisis total, and `:16636` — "Virtues that affect aging rolls do not
/// affect crisis survival rolls" — cuts the same way for the roll that follows.
/// (That sentence is strictly about the *survival* roll, which is its own
/// read-out; it is quoted here because the temptation is to fold
/// [`AgingEffect::AgingRoll`] into this total by analogy with [`aging_total`].
/// Do not: the analogy is exactly what the book denies.)
///
/// # The Decrepitude is the year's, not today's
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on
/// > the Crisis Table. (`:16619`)
///
/// "First" fixes the score this total adds: the one the crisis year itself
/// raised. Reading [`decrepitude_score`] live would be right only if the crisis
/// year were the newest year applied — and it need not be. [`resolve_year`]
/// refuses nothing but a year already recorded, and this module is deliberately
/// order-independent, so a player may roll 36 (which flags a Crisis), carry on
/// through 37-40, and resolve 36's Crisis afterwards. A live read would then
/// charge that Crisis with four later years' Aging Points. So the score comes
/// from [`decrepitude_points_as_of`] instead.
///
/// # The die is not policed
///
/// "Roll a ten-sided die. Each number counts for its value, except that a zero
/// counts as ten." (`:474`) bounds what a UI should *offer*; it is an input
/// affordance, not an engine rule. A storyguide may hand out any number, so a
/// die outside 1..=10 is totalled as given rather than refused.
///
/// `None` when the ruleset ships no aging rules, or aging rules with no Crisis
/// Table — the engine never invents a table the ruleset does not carry.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16619, :16621,
/// :16636, :474.
pub fn crisis_total(entity: &Entity, ruleset: &Ruleset, age: u32, die: i32) -> Option<CrisisTotal> {
    let rules = ruleset.aging()?;
    // A ruleset with no Crisis Table has no crisis to total, exactly as one with
    // no aging rules has no year to roll.
    rules.crisis.as_ref()?;

    let age_modifier = rules.age_modifier(age);
    let score = ruleset
        .advancement()
        .score_for_xp(decrepitude_points_as_of(entity, age));

    Some(CrisisTotal {
        age,
        die,
        age_modifier,
        decrepitude_score: score,
        total: die + age_modifier + i32::from(score),
    })
}

/// The Aging Points that had reached Decrepitude by the end of the character's
/// `age`th year — his lifetime total less every point a **later** year awarded.
///
/// This is what `:16619`'s "Increase the character's Decrepitude first, and then
/// roll on the Crisis Table" needs and a live [`decrepitude_points_total`] read
/// does not give, because years may be resolved in any order (see
/// [`crisis_total`]).
///
/// Two boundaries decide the arithmetic:
///
/// - **Strictly greater.** The crisis year's own entry carries `age == Some(age)`,
///   which is not greater, so the points that year awarded stay IN — they are
///   precisely the increase `:16619` puts first.
/// - **Undated entries never subtract.** A legacy hand-written entry carries
///   neither an `age` nor a distribution, so there is nothing to date it by and
///   nothing to take off; such a character rolls against every point he has.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16619, :16617.
fn decrepitude_points_as_of(entity: &Entity, age: u32) -> u32 {
    let later: u32 = entity
        .aging_log
        .iter()
        .filter(|entry| entry.age.is_some_and(|logged| logged > age))
        .flat_map(|entry| entry.points.values())
        .map(|points| u32::from(*points))
        .sum();
    decrepitude_points_total(entity).saturating_sub(later)
}

/// Which row of the Crisis Table a CRISIS TOTAL lands on (`:16624-16632`).
///
/// The Aging Roll table's [`resolve_outcome`] twin, and deliberately the simpler
/// of the two: an aging row has to be read against the character before it means
/// anything ("sufficient Aging Points … to reach the next level in Decrepitude",
/// `:16602`), while a crisis row already says everything it does. So this takes
/// no [`Entity`] and hands back the row itself — the caller needs its [`Id`] as
/// much as its [`CrisisOutcome`], because the row's display text ("Bedridden for
/// a week") lives in `rules/i18n/<lang>/aging.json` keyed by that id and never in
/// the engine.
///
/// # Every total lands somewhere
///
/// The table's first row is open below ("8 or less", `:16626`) and its last open
/// above ("19+", `:16632`), and `Ruleset::validate_crisis_rules` refuses at load
/// any table whose rows leave a gap or overlap between those ends. So for a
/// ruleset that loaded, a `None` here means the ruleset ships **no Crisis Table**
/// — not that the total fell off the table.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16624-16632.
pub fn resolve_crisis_row(ruleset: &Ruleset, total: i32) -> Option<&CrisisRow> {
    ruleset
        .aging()?
        .crisis
        .as_ref()?
        .rows
        .iter()
        .find(|row| row.covers(total))
}

/// What surviving one Crisis would take, and what the character brings to it.
///
/// A **read-out, never a resolution.** It reports the Ease Factor of the Stamina
/// stress roll (`:16628-16631`), the level of the Momentary Creo Corpus Ritual
/// that resolves the crisis instead (`:16638`), every survival modifier the
/// character carries, and what the rules *allow* someone else to contribute
/// (`:16634`). It then stops. See [`crisis_survival`] for why.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16628-16638.
// `Serialize` only, like every other aging read-out: computed on demand from the
// entity and the ruleset, never read back off a save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrisisSurvival {
    /// The Ease Factor of the Stamina stress roll the row calls for — 3 at Minor
    /// up to 12 at Critical (`:16628-16631`).
    ///
    /// `None` for Terminal illness, which offers **no roll at all**: "CrCo40
    /// required to survive." (`:16632`) An absent Ease Factor is not an
    /// unbeatable one; there is simply no Stamina roll to make, so
    /// [`Self::modifier_total`] has nothing to modify and only
    /// [`Self::ritual_level`] can answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ease_factor: Option<i32>,
    /// The level of the Momentary Creo Corpus Ritual that resolves the crisis:
    /// "The level of spell required depends on the severity of the crisis, as
    /// noted on the table." (`:16638`) — 20 at Minor up to 40 at Terminal.
    pub ritual_level: u32,
    /// Every survival modifier the character carries, **itemized**: the UI has to
    /// be able to name each one, and a pre-summed number names none. Each carries
    /// its source rather than a label, because a raw [`Id`] is never rendered —
    /// the frontend resolves it through its display-name lookup.
    pub modifiers: Vec<CrisisModifier>,
    /// The sum of [`Self::modifiers`]. Both are reported because the sheet shows
    /// the breakdown *and* the number, and a caller must never have to re-add the
    /// terms to get the second.
    pub modifier_total: i32,
    /// What the rules **allow** someone else to bring, which is not the same
    /// thing as a modifier the character has: the attending doctor of `:16634`.
    /// The engine cannot score it, because the Medicine belongs to a character
    /// this sheet does not hold.
    pub allowances: Vec<CrisisAllowance>,
}

/// One modifier to the crisis survival roll, named by where it comes from.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :10844.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrisisModifier {
    /// Where it comes from.
    pub source: CrisisModifierSource,
    /// How much, **with its stored sign** and ADDED — the one convention this
    /// file keeps (see [`crisis_survival`]).
    pub amount: i32,
}

/// Where a [`CrisisModifier`] comes from.
///
/// A tagged enum rather than a string, so a new source is a compile error at
/// every reader until it is handled, and so the UI can render the two cases
/// differently: a Virtue resolves through the item catalogue, the cord does not.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :10844.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CrisisModifierSource {
    /// A Virtue or Flaw the character carries that grants a bonus to the survival
    /// roll by name — Mild Aging's "+3 bonus to rolls to survive an aging crisis"
    /// (`:4530`).
    ///
    /// Carries the **id**, never a name: a raw [`Id`] is never a user-facing
    /// label, so the frontend resolves it through its display-name lookup like
    /// every other item reference.
    Trait {
        /// The item's id.
        item: Id,
    },
    /// The familiar's Bronze cord: "You can apply your bronze cord score as a
    /// bonus to … rolls to resist aging." (`:10844`) The roll that names is this
    /// one — an aging roll is not a roll one passes or fails — and `:16636` keeps
    /// the two families apart, so the cord reaches this total and never the
    /// AGING TOTAL.
    BronzeCord,
}

/// Something the rules **permit** at a crisis, as opposed to a number the engine
/// adds.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16634.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CrisisAllowance {
    /// The attending doctor: "An Int + Medicine roll against an Ease Factor of 6
    /// allows the character to add the attendant's Medicine score to the roll to
    /// survive the crisis. Only one doctor may usefully attend a patient, and if
    /// the doctor botches the character must subtract 3 from the survival roll."
    /// (`:16634`)
    ///
    /// Reported rather than scored: the Medicine score belongs to *another*
    /// character, whom this sheet does not hold, so the app can state what is
    /// allowed and no more. "Only one doctor" is why this is at most one entry.
    Attendant {
        /// The Ability rolled, and added on a success — `ability.medicine`.
        ability: Id,
        /// The Characteristic added to the attendant's own roll — Int.
        characteristic: Characteristic,
        /// The Ease Factor the attendant's roll must beat: 6.
        ease_factor: i32,
        /// What a botch costs the patient, stored **signed and added** like every
        /// other modifier in this file: -3.
        botch_penalty: i32,
    },
}

/// What surviving `outcome` would take, and what this character brings to it
/// (`:16628-16638`).
///
/// # It computes; it never rolls
///
/// The engine reports the Ease Factor, the Creo Corpus level that resolves the
/// crisis instead, the modifiers the character carries, and the doctor's
/// allowance — and then stops. It **never throws the Stamina die, never says a
/// character survived or died, and never kills one**. `arm-rules` has no `rand`
/// dependency and never will: a generator that rolled for the player would
/// invent rules-relevant state no one at the table agreed to, and death is the
/// one outcome a character sheet must never author on its own.
///
/// # `None` means Bedridden
///
/// "Bedridden for a week" (`:16626`) and "Bedridden for a month." (`:16627`) are
/// time, not a roll: no Stamina roll, no Ritual level, nothing for a modifier to
/// modify. There is no survival read-out to give, so there is none — rather than
/// an empty one that would read as "survivable on a 0".
///
/// # `:16636` is the whole point
///
/// > Virtues that affect aging rolls do not affect crisis survival rolls.
///
/// So [`AgingEffect::AgingRoll`] and [`AgingEffect::LivingConditions`] amounts
/// never reach [`CrisisSurvival::modifier_total`]. Only two things do: an
/// [`AgingEffect::CrisisSurvival`] amount, which is a grant to *this* roll by
/// name, and the Bronze cord (`:10844`). Mild Aging is the case that proves the
/// wall is load-bearing, because `:4530` grants both kinds in one sentence —
/// "The character's aging rolls benefit from a +1 bonus to the Living Conditions
/// Modifier … Furthermore, he receives a +3 bonus to rolls to survive an aging
/// crisis." The +1 stays on the aging roll; only the +3 arrives here.
///
/// # Signs
///
/// Every modifier is ADDED with its stored sign — the one convention this file
/// keeps, in which only the terms a rule *names* as subtracted are subtracted
/// (`:16567-16569` names two, and this roll's rules name none). That includes the
/// attendant's `botch_penalty`, which ships as -3.
///
/// # Itemized, and summed
///
/// [`CrisisSurvival::modifiers`] carries each term separately so the UI can name
/// it, and [`CrisisSurvival::modifier_total`] carries the sum so no caller has to
/// re-add them. A modifier is listed when the character actually carries its
/// source, so a magus with no familiar shows no cord line at all rather than a
/// "+0".
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:4530, :10844, :16626,
/// :16627, :16628-16632, :16634, :16636, :16638.
pub fn crisis_survival(
    entity: &Entity,
    ruleset: &Ruleset,
    outcome: &CrisisOutcome,
) -> Option<CrisisSurvival> {
    // Bedridden is time, not a roll (`:16626`, `:16627`) — nothing to describe.
    let CrisisOutcome::Illness {
        ease_factor,
        ritual_level,
        ..
    } = outcome
    else {
        return None;
    };

    let mut modifiers = Vec::new();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            let Effect::AgingMod { kind, amount } = effect else {
                continue;
            };
            match kind {
                // A grant to this roll by name — Mild Aging's +3 (`:4530`).
                AgingEffect::CrisisSurvival => modifiers.push(CrisisModifier {
                    source: CrisisModifierSource::Trait {
                        item: selection.item_ref.clone(),
                    },
                    amount: i32::from(*amount),
                }),
                // "Virtues that affect aging rolls do not affect crisis survival
                // rolls." (`:16636`) — the two kinds the AGING TOTAL takes are
                // walled off from this roll, and this arm is the wall. The
                // remaining kinds belong to neither roll: `longevity_bonus`
                // modifies a ritual bonus, `no_aging` / `no_apparent_aging` are
                // exemptions rather than numbers, `decrepitude` moves the accrued
                // score, and `crisis_heavy_wound` is a consequence of a crisis
                // (`:6340`) rather than a term of the roll to survive one.
                AgingEffect::AgingRoll
                | AgingEffect::LivingConditions
                | AgingEffect::LongevityBonus
                | AgingEffect::NoAging
                | AgingEffect::NoApparentAging
                | AgingEffect::Decrepitude
                | AgingEffect::CrisisHeavyWound => {}
            }
        }
    }

    // "and to rolls to resist aging" (`:10844`), through the one entity-level
    // accessor, so the +5 cord maximum (`:10836`) keeps its single home.
    let bronze_cord = crate::derived::bronze_cord_bonus(entity);
    if bronze_cord != 0 {
        modifiers.push(CrisisModifier {
            source: CrisisModifierSource::BronzeCord,
            amount: bronze_cord,
        });
    }

    let modifier_total = modifiers.iter().map(|modifier| modifier.amount).sum();

    // "Only one doctor may usefully attend a patient" (`:16634`), so at most one
    // — and none at all from a ruleset that ships no attendant.
    let allowances = ruleset
        .aging()
        .and_then(|rules| rules.crisis.as_ref())
        .and_then(|crisis| crisis.attendant.as_ref())
        .map(|attendant| CrisisAllowance::Attendant {
            ability: attendant.ability.clone(),
            characteristic: attendant.characteristic,
            ease_factor: attendant.ease_factor,
            botch_penalty: attendant.botch_penalty,
        })
        .into_iter()
        .collect();

    Some(CrisisSurvival {
        ease_factor: *ease_factor,
        ritual_level: *ritual_level,
        modifiers,
        modifier_total,
        allowances,
    })
}

/// One Crisis, read whole: the total, the row it lands on, what the row costs,
/// and what surviving it would take.
///
/// The four values a caller needs are composed here rather than left to be
/// fetched one at a time, for the reason `AgingProjection` composes the AGING
/// TOTAL with its row: a UI that assembled them itself would be free to pair a
/// total with the wrong row, and there is exactly one right pairing.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16621-16638.
// `Serialize` only, like every other aging read-out: computed on demand from the
// entity, the ruleset, the year and the die, and never read back off a save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrisisPreview {
    /// The CRISIS TOTAL and every term that made it (`:16621`).
    pub total: CrisisTotal,
    /// The **id** of the row the total landed on — `crisis.minor_illness` and
    /// friends. An id, never a name: the row's display text lives in
    /// `rules/i18n/<lang>/aging.json` keyed by this, so the frontend resolves it
    /// there like every other rules string.
    pub row: Id,
    /// What that row costs the character (`:16624-16632`).
    pub outcome: CrisisOutcome,
    /// What surviving it would take (`:16628-16638`), when the row calls for a
    /// roll at all. Absent for [`CrisisOutcome::Bedridden`], which is time rather
    /// than a roll (`:16626`, `:16627`) — see [`crisis_survival`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survival: Option<CrisisSurvival>,
}

/// Everything the engine can say about one Crisis, from the character, the year
/// it fell in and the Simple Die the player typed.
///
/// # Composed, not decided
///
/// [`crisis_total`], [`resolve_crisis_row`] and [`crisis_survival`] each stay the
/// single home of their own rule; this only pairs them, which is the one thing a
/// caller must not be left to do. The pairing is what makes the three consistent:
/// the row is looked up against the total this call computed, and the survival
/// read-out against the outcome that row carries.
///
/// # It writes nothing, and it never rolls
///
/// The Crisis is a **reading**. No Aging Point moves, no log entry appears, no
/// Decrepitude is raised — `resolve_year` remains the aging subsystem's single
/// writer, and `:16619`'s "increase the character's Decrepitude first" is
/// honoured by reading the score as of the crisis year (see [`crisis_total`])
/// rather than by writing anything here. The Stamina die is never thrown and no
/// character is ever pronounced dead: `arm-rules` has no `rand` dependency, and
/// death is the one outcome a character sheet must not author on its own.
///
/// `None` when the ruleset ships no aging rules, or aging rules with no Crisis
/// Table.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16638.
pub fn crisis_preview(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
) -> Option<CrisisPreview> {
    let total = crisis_total(entity, ruleset, age, die)?;
    let row = resolve_crisis_row(ruleset, total.total)?;
    let survival = crisis_survival(entity, ruleset, &row.outcome);

    Some(CrisisPreview {
        total,
        row: row.id.clone(),
        outcome: row.outcome.clone(),
        survival,
    })
}

/// One year's aging roll as the player submits it.
///
/// The engine never rolls: `die` is the stress die (no botch) thrown at the
/// table and typed in (`:16567`). `age` is the age the roll is made at rather
/// than the character's current age, because `:2232`'s pre-play catch-up walks
/// every owed year and each uses that year's own age.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567, :16615.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgingYearRequest {
    /// The character's age in the year being resolved.
    pub age: u32,
    /// The stress die the player rolled and typed in.
    pub die: i32,
    /// Where the points the row leaves to the player go, per Characteristic:
    /// "If an Aging Point 'in any Characteristic' is gained, the player may
    /// choose the Characteristic" (`:16615`). A **map** rather than a single
    /// pick, because reaching the next Decrepitude level asks for points "in any
    /// Characteristic**s**" (`:16602`, `:16611`) and forcing them all onto one
    /// would force Characteristic drops the player may legally avoid. Empty for
    /// a row that names its own Characteristics, and for one that awards
    /// nothing.
    pub distribution: BTreeMap<Characteristic, u8>,
    /// The **Simple Die** the player threw at the Crisis Table (`:16621`), when
    /// the year's row sent him there and he has rolled it. The engine never rolls
    /// this one either.
    ///
    /// `None` is a Crisis the table demanded and nobody has rolled yet, which is a
    /// legitimate state rather than a refusal: the aging roll happened whether or
    /// not the second die has been thrown, and refusing to record it would lose
    /// the one thing that did. A die given for a year the table sent to no Crisis
    /// is simply unused — whether a Crisis happened is `:16602`/`:16611`'s call,
    /// never the player's.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16619, :16621.
    pub crisis_die: Option<i32>,
}

/// What resolving one year produced: the character it made, and the reading that
/// made it.
///
/// The total and the outcome come back with the entity so a caller can show the
/// arithmetic it just applied without recomputing it against an entity that has
/// since changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgingYearResult {
    /// The character after the year — a new value; the caller's is untouched.
    pub entity: Entity,
    /// The AGING TOTAL that was rolled, every term included.
    pub total: AgingTotal,
    /// What the table did with it.
    pub outcome: AgingOutcome,
    /// The Crisis the year sent the character to, read whole (`:16619-16638`) —
    /// present only when [`AgingOutcome::crisis`] is set **and** the request
    /// carried a [`AgingYearRequest::crisis_die`] **and** the ruleset ships a
    /// Crisis Table. Otherwise the Crisis is owed and unrolled, which the log
    /// records as `crisis` with no row.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16638.
    pub crisis: Option<CrisisPreview>,
    /// What the year changed about the character that the character itself cannot
    /// show — today, only the Longevity Ritual a Crisis spends (`:16573`). Empty
    /// for almost every year.
    pub notes: Vec<AgingNote>,
}

/// Something a resolved year has to **tell** the player, as opposed to something
/// it writes.
///
/// A tagged enum rather than a message, for the reason every refusal in this file
/// is plain data: the engine hardcodes no user-facing string, so the caller maps
/// each variant through Fluent. An exhaustive `match` also makes a new note a
/// compile error at every reader until it has been rendered somewhere.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16573.
// `Serialize` so the note can cross the IPC edge unchanged; never deserialized,
// because it is produced by a single call and stored by nobody.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgingNote {
    /// The Crisis this year suffered spent the character's Longevity Ritual.
    ///
    /// > A Longevity Ritual is effective until the character suffers a crisis.
    /// > When the crisis occurs, the ritual assures that the character survives,
    /// > but its power is spent, and the focal ritual must be performed again.
    /// > (`:16573`)
    ///
    /// **Reported, never applied.** [`resolve_year`] leaves
    /// [`Entity::longevity_ritual`] exactly as it found it: the entry is a stored
    /// choice holding a player-entered bonus and the focus that "must be repeated"
    /// if the ritual is performed again (`:10668`), and an engine that silently
    /// deleted it would destroy both — and make the year unrevertible. Performing
    /// the focal ritual again is a season's work the player records; the sheet does
    /// not infer it.
    ///
    /// It follows the **Crisis**, not the Crisis roll: "when the crisis occurs" is
    /// the aging row's doing (`:16602`, `:16611`), and the Simple Die only decides
    /// how bad it was — so a Crisis owed and unrolled spends the ritual too.
    LongevityRitualSpent,
}

/// Why a year could not be resolved, or reverted.
///
/// Plain data: no issue codes, no Fluent keys, no user-facing prose — the same
/// idiom as [`ChildhoodRejection`](crate::childhood::ChildhoodRejection).
/// Turning one into a localized message is the caller's job.
///
/// Every variant is a **refusal to write**, never a silent adjustment. A year
/// the engine cannot apply exactly is a year the player must be told about,
/// because the alternative is a character sheet that quietly disagrees with the
/// dice that were rolled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgingError {
    /// The ruleset ships no aging rules, so there is no table to roll against.
    /// An absent block stands the whole subsystem down rather than letting the
    /// engine invent a threshold or a row.
    NoAgingRules,
    /// The log already carries a resolved entry for this age. Applying it again
    /// would charge the character twice for one roll.
    YearAlreadyRecorded {
        /// The age already recorded.
        age: u32,
    },
    /// The distribution does not sum to the points the row left to the player.
    /// The count is the table's, not the player's: "sufficient Aging Points … to
    /// reach the next level in Decrepitude" (`:16602`).
    DistributionMismatch {
        /// The points the row leaves to the player to place.
        owed: u32,
        /// What the request actually distributed.
        distributed: u32,
    },
    /// The row names its own Characteristics (`:16603-16610`) — or awards
    /// nothing at all — so there was nothing for the player to place, yet the
    /// request carried a distribution. Applying it would invent points the table
    /// never awarded.
    DistributionNotOpen {
        /// The Characteristics the request tried to place points in, in
        /// canonical order.
        characteristics: Vec<Characteristic>,
    },
    /// The row asks for "sufficient Aging Points … to reach the next level in
    /// Decrepitude" (`:16602`) and the advancement curve cannot price that
    /// level — Decrepitude "increases as an Ability" (`:16617`) and the table
    /// tops out. Reported as unpriceable rather than silently costed at 0.
    AwardUnpriceable,
    /// No log entry records this age, so there is nothing to revert. A
    /// hand-written free-text entry carries no age and is out of
    /// [`revert_year`]'s reach by construction — see its doc.
    YearNotRecorded {
        /// The age that resolved to no entry.
        age: u32,
    },
}

/// Resolves one year's aging roll and **applies** it — the single writer in the
/// aging subsystem.
///
/// It computes the year's [`AgingTotal`], reads it against the table with
/// [`resolve_outcome`], and returns the character that results: the awarded
/// Aging Points added to [`Entity::aging_points`], the apparent age advanced if
/// the total cleared the threshold (`:16577`), and a structured
/// [`AgingLogEntry`] appended.
///
/// # What it does not do
///
/// It **never rolls** — both dice are the player's, typed in. It **never
/// kills**: a Crisis is read, recorded and stopped at, because whether the
/// character survives is the table's to decide (see [`crisis_survival`]). And it
/// **never touches Decrepitude**: "Every Aging Point also counts as an
/// experience point towards Decrepitude, which increases as an Ability"
/// (`:16617`), so the score follows from the points through
/// [`decrepitude_score`] and is never written down beside them.
///
/// # The Crisis leg, and why the order matters
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on
/// > the Crisis Table. (`:16619`)
///
/// A row carrying a Crisis (`:16602`, `:16611`) awards its Aging Points like any
/// other, and **that award is the increase `:16619` puts first**. The Crisis is
/// then read off the character those points already made — never off the one who
/// walked into the year — so the CRISIS TOTAL adds the Decrepitude this very year
/// raised. [`crisis_preview`] does the reading; this only orders it.
///
/// The Crisis is resolved when the player has thrown the Simple Die and the
/// ruleset ships a Crisis Table. Otherwise the year is still written, with the
/// Crisis recorded as **owed and unrolled**
/// ([`AgingLogEntry::crisis`](crate::types::AgingLogEntry::crisis) set and no
/// row) — the aging roll happened whether or not the second die has been thrown.
///
/// # A spent Longevity Ritual is reported, never deleted
///
/// "A Longevity Ritual is effective until the character suffers a crisis. When
/// the crisis occurs, the ritual assures that the character survives, but its
/// power is spent, and the focal ritual must be performed again" (`:16573`). The
/// year says so with an [`AgingNote::LongevityRitualSpent`] and leaves
/// [`Entity::longevity_ritual`] exactly where it found it — see the variant for
/// why deleting a stored choice is the wrong half of that sentence to implement.
///
/// The entity is not mutated in place, so a refused year leaves the caller's own
/// character exactly as it was — and [`revert_year`] can put an applied one
/// back.
///
/// # The log entry records, it does not derive
///
/// [`AgingLogEntry::effect`] is left empty: the structured fields *are* the
/// record, and the prose is the player's to add later. The entry keeps the
/// Living Conditions and the total the year was rolled under because they are
/// **history** — the character's standing conditions and ritual may legitimately
/// change afterwards, and the roll that was made does not change with them.
///
/// # Seeding the apparent age
///
/// [`Entity::apparent_age`] is `None` on most characters. The first year that
/// needs it seeds it at [`AgingRules::start_age`] — nothing has happened to the
/// appearance before the first owed roll — and every year after that only
/// increments. Seeding once and incrementing thereafter is what makes the result
/// **order-independent**: applying a catch-up's years out of order lands on the
/// same apparent age as applying them in order. A hand-entered apparent age is
/// never re-seeded, only advanced.
///
/// A character carrying [`AgingEffect::NoApparentAging`] (`:3488`, `:5189`) is
/// neither seeded nor advanced, because [`resolve_outcome`] has already reported
/// that his appearance does not follow the roll — one decision, read here rather
/// than made twice.
///
/// # Refusals
///
/// Every [`AgingError`] here is a refusal to write; nothing is ever partially
/// applied or silently dropped. See the variants for the five cases.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16617.
pub fn resolve_year(
    entity: &Entity,
    ruleset: &Ruleset,
    request: &AgingYearRequest,
) -> Result<AgingYearResult, AgingError> {
    let Some(rules) = ruleset.aging() else {
        return Err(AgingError::NoAgingRules);
    };
    if logged_year(entity, request.age).is_some() {
        return Err(AgingError::YearAlreadyRecorded { age: request.age });
    }

    // Both read the character as it stands *before* the year: the next
    // Decrepitude level is priced off the points already accrued.
    let (Some(total), Some(outcome)) = (
        aging_total(entity, ruleset, request.age, request.die),
        aging_total(entity, ruleset, request.age, request.die)
            .and_then(|total| resolve_outcome(entity, ruleset, total.total)),
    ) else {
        return Err(AgingError::NoAgingRules);
    };

    let awarded = award_points(&outcome, &request.distribution)?;

    let mut applied = entity.clone();
    for (characteristic, points) in &awarded {
        let accrued = applied.aging_points.entry(*characteristic).or_insert(0);
        *accrued = accrued.saturating_add(*points);
    }
    if outcome.apparent_age_increases {
        let seeded = applied.apparent_age.unwrap_or(rules.start_age);
        applied.apparent_age = Some(seeded.saturating_add(1));
    }

    // "**Crisis:** Increase the character's Decrepitude first, and then roll on
    // the Crisis Table." (`:16619`) — the award above IS that increase, and
    // reading the Crisis off `applied` rather than off `entity` is what puts it
    // first. `crisis_total` measures the score as of this year, so the points
    // just awarded count and any later year's do not.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:16619.
    let crisis = outcome
        .crisis
        .then_some(request.crisis_die)
        .flatten()
        .and_then(|die| crisis_preview(&applied, ruleset, request.age, die));

    applied.aging_log.push(AgingLogEntry {
        year: calendar_year(entity, request.age),
        age: Some(request.age),
        effect: String::new(),
        die: Some(request.die),
        total: Some(total.total),
        living_conditions: entity.living_conditions.clone(),
        points: awarded,
        apparent_age_increased: outcome.apparent_age_increases,
        crisis: outcome.crisis,
        crisis_die: crisis.as_ref().map(|crisis| crisis.total.die),
        crisis_total: crisis.as_ref().map(|crisis| crisis.total.total),
        crisis_row: crisis.as_ref().map(|crisis| crisis.row.clone()),
        crisis_severity: crisis.as_ref().and_then(|crisis| match crisis.outcome {
            CrisisOutcome::Illness { severity, .. } => Some(severity),
            CrisisOutcome::Bedridden => None,
        }),
    });
    applied.normalize();

    // "A Longevity Ritual is effective until the character suffers a crisis. When
    // the crisis occurs, the ritual assures that the character survives, but its
    // power is spent" (`:16573`) — said, and not done: the ritual is the player's
    // stored choice and stays on the entity untouched.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:16573.
    let notes = (outcome.crisis && entity.longevity_ritual.is_some())
        .then_some(AgingNote::LongevityRitualSpent)
        .into_iter()
        .collect();

    Ok(AgingYearResult {
        entity: applied,
        total,
        outcome,
        crisis,
        notes,
    })
}

/// Undoes one resolved year, exactly: the points it awarded come back off, the
/// apparent age steps back if that year advanced it, and the entry is removed.
///
/// # Why this exists
///
/// A magus of 60 owes 25 aging rolls before play begins (`:2232`), and a
/// 25-roll walk with no undo is not shippable — a mistyped die has to be
/// recoverable. It is *exact* rather than a recomputation because the widened
/// [`AgingLogEntry`] records precisely what its year did: the points it placed,
/// and whether the appearance moved. Re-deriving them would get a different
/// answer whenever the character's Living Conditions, Longevity Ritual or
/// accrued points have changed since — which they legitimately may.
///
/// # It addresses entries by age
///
/// [`AgingLogEntry::age`] is the key, because a character with no `birth_year`
/// has no calendar year to be addressed by. A **legacy free-text entry carries
/// no age**, so it is out of this function's reach by construction and is
/// removed with the ordinary log editor instead — which is the right home for
/// it, since there is nothing mechanical to undo.
///
/// # Undoing the seed
///
/// [`resolve_year`] seeds an unset [`Entity::apparent_age`] at
/// [`AgingRules::start_age`] before advancing it, so the exact inverse clears
/// the field again when the decrement lands back on that age: a character whose
/// appearance sits at the age before the first owed roll has an apparent age no
/// roll has moved, which is what `None` means. The one thing this collapses is a
/// hand-entered apparent age of exactly `start_age`, which comes back unset —
/// the two states carry the same fact and the entry cannot tell them apart.
/// Under a ruleset shipping no aging rules there is no start age to compare, so
/// the decremented figure simply stays.
///
/// Reverting an age no entry records is [`AgingError::YearNotRecorded`], never a
/// silent no-op: a revert that quietly did nothing would tell the player the
/// year had been undone.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16577-16617.
pub fn revert_year(entity: &Entity, ruleset: &Ruleset, age: u32) -> Result<Entity, AgingError> {
    let Some(entry) = logged_year(entity, age) else {
        return Err(AgingError::YearNotRecorded { age });
    };

    let mut reverted = entity.clone();
    for (characteristic, points) in &entry.points {
        let Some(accrued) = reverted.aging_points.get_mut(characteristic) else {
            continue;
        };
        *accrued = accrued.saturating_sub(*points);
        // The writer never leaves a zero entry, so neither does its inverse.
        if *accrued == 0 {
            reverted.aging_points.remove(characteristic);
        }
    }

    if entry.apparent_age_increased
        && let Some(apparent) = reverted.apparent_age
    {
        let stepped_back = apparent.saturating_sub(1);
        let unmoved = ruleset
            .aging()
            .is_some_and(|rules| stepped_back == rules.start_age);
        reverted.apparent_age = (!unmoved).then_some(stepped_back);
    }

    reverted.aging_log.retain(|logged| logged.age != Some(age));
    reverted.normalize();
    Ok(reverted)
}

/// Where the year's Aging Points land, per Characteristic: the ones the row
/// names itself, plus the ones the player placed.
///
/// A zero is not an award, so it is never written — which keeps the log entry
/// canonical and makes [`revert_year`] the exact inverse.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16601-16615.
fn award_points(
    outcome: &AgingOutcome,
    distribution: &BTreeMap<Characteristic, u8>,
) -> Result<BTreeMap<Characteristic, u8>, AgingError> {
    let mut awarded: BTreeMap<Characteristic, u8> = BTreeMap::new();
    let mut owed = 0u32;
    for award in &outcome.awards {
        // `None` is the unpriceable next Decrepitude level, which cannot be
        // costed at 0 without quietly awarding the character nothing.
        let Some(points) = award.points else {
            return Err(AgingError::AwardUnpriceable);
        };
        match award.target {
            AgingPointTarget::Named(characteristic) => add_points(
                &mut awarded,
                characteristic,
                u8::try_from(points).unwrap_or(u8::MAX),
            ),
            AgingPointTarget::PlayerChoice | AgingPointTarget::NextDecrepitudeLevel => {
                owed = owed.saturating_add(points);
            }
        }
    }

    if owed == 0 {
        if !distribution.is_empty() {
            return Err(AgingError::DistributionNotOpen {
                characteristics: distribution.keys().copied().collect(),
            });
        }
        return Ok(awarded);
    }

    let distributed: u32 = distribution.values().map(|points| u32::from(*points)).sum();
    if distributed != owed {
        return Err(AgingError::DistributionMismatch { owed, distributed });
    }
    for (characteristic, points) in distribution {
        add_points(&mut awarded, *characteristic, *points);
    }
    Ok(awarded)
}

/// Adds `points` to a Characteristic's entry, leaving a zero unwritten.
fn add_points(
    awarded: &mut BTreeMap<Characteristic, u8>,
    characteristic: Characteristic,
    points: u8,
) {
    if points == 0 {
        return;
    }
    let entry = awarded.entry(characteristic).or_insert(0);
    *entry = entry.saturating_add(points);
}

/// What a Longevity Ritual does to the roll of a character too young to be
/// aging yet.
///
/// > A character under the influence of a Longevity Ritual should roll on the
/// > table no matter what his age, but treats all rolls of 10 or more as rolls
/// > of 9 until he reaches the age of 35.
///
/// So the young ritual-bearer rolls, and his apparent age may creep up, but the
/// clamp keeps him off every row that costs an Aging Point: "he is at no risk of
/// actually aging before any other characters."
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16575.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongevityClamp {
    /// The total every higher total is treated as ("rolls of 10 or more as rolls
    /// of 9" — the 9, not the 10, because it is the value the roll becomes).
    pub max_total: i32,
    /// The age the clamp stops at ("until he reaches the age of 35").
    pub until_age: u32,
}

/// One row of the Living Conditions table: how a character's circumstances
/// modify the aging total.
///
/// A higher modifier means a longer life — "a high Longevity Ritual modifier and
/// a high Living Conditions modifier both indicate longer life" (`:16571`) —
/// because the modifier is *subtracted* from the total.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16581-16594.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivingCondition {
    /// Slug-style id, e.g. `living_condition.average_peasant`. Its display name
    /// lives in `rules/i18n`, keyed by this id.
    pub id: Id,
    /// The modifier the row contributes, from `+2` down to `-2`.
    pub modifier: i8,
    /// Whether the row stacks with the other cumulative ones: "Modifiers marked
    /// with an asterisk are cumulative with each other" (`:16594`). The
    /// non-cumulative rows are alternatives, so at most one of them applies.
    #[serde(default, skip_serializing_if = "is_false")]
    pub cumulative: bool,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// One row of the Aging Roll table: a band of totals and what landing in it
/// costs.
///
/// The band is inclusive on both ends, and `max` is absent for the open-ended
/// top row ("22+", `:16611`). The two "apparent aging" rows of `:16599-16600`
/// are **not** rows here — they are not alternatives to the rest but a separate
/// question asked of every total, which is what
/// [`AgingRules::apparent_age_increase_min`] answers.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16597-16611.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingRow {
    /// Lowest total the row covers (inclusive).
    pub min: i32,
    /// Highest total the row covers (inclusive). `None` for the open-ended top
    /// row, which has no upper bound at all rather than a very large one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    /// What the row costs the character.
    pub effect: AgingRowEffect,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl AgingRow {
    /// Whether `total` lands on this row: the band is inclusive on both ends, and
    /// an absent [`Self::max`] is the open-ended "22+" top row (`:16611`), which
    /// has no upper bound at all rather than a very large one.
    fn covers(&self, total: i32) -> bool {
        self.min <= total && self.max.is_none_or(|max| total <= max)
    }
}

/// What an [`AgingRow`] does to the character.
///
/// A tagged enum rather than free-text, so a kind the engine cannot apply is a
/// load-time failure instead of a silently ignored row — and so adding a kind is
/// a compile error until every reader has decided what to do with it.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16597-16611.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgingRowEffect {
    /// Aging Points the player places where they like: "1 Aging Point in any
    /// Characteristic" (`:16601`), since "If an Aging Point 'in any
    /// Characteristic' is gained, the player may choose the Characteristic"
    /// (`:16615`).
    AnyCharacteristic {
        /// How many points the row gives.
        points: u32,
    },
    /// Aging Points in Characteristics the row names — one ("1 Aging Point in
    /// Qik", `:16603`) or two ("1 Aging Point in Str and Sta", `:16607`). The
    /// points are per named Characteristic, not divided between them.
    NamedCharacteristics {
        /// How many points each named Characteristic gets.
        points: u32,
        /// The Characteristics the row names, in file order.
        characteristics: Vec<Characteristic>,
    },
    /// "Gain sufficient Aging Points (in any Characteristics) to reach the next
    /// level in Decrepitude, and Crisis" (`:16602`, `:16611`). Resolving the
    /// crisis itself (`:16619-16632`) is a later slice; this variant only
    /// records that one is owed.
    NextDecrepitudeLevelAndCrisis,
}

/// The Crisis Table and the two rolls around it, loaded as part of
/// `rules/core/aging.json`.
///
/// "**Crisis:** Increase the character's Decrepitude first, and then roll on the
/// Crisis Table." (`:16619`) — **CRISIS TOTAL: Simple die + age/10 (round up) +
/// Decrepitude Score** (`:16621`).
///
/// [`Self::die`] and [`Self::attendant`] are optional because a ruleset may ship
/// the table without them; the rows are not, since a crisis table with no rows
/// resolves nothing.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16634.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrisisRules {
    /// The Crisis Table (`:16624-16632`), in file order.
    pub rows: Vec<CrisisRow>,
    /// The die the crisis total is rolled on (`:474`), when the ruleset ships it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub die: Option<CrisisDie>,
    /// The attending doctor of `:16634`, when the ruleset ships them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attendant: Option<CrisisAttendant>,
}

/// One row of the Crisis Table: a band of crisis totals and what landing in it
/// costs.
///
/// The band is inclusive on both ends and **both** ends may be open — "8 or
/// less" (`:16626`) has no lower bound and "19+" (`:16632`) no upper one. That
/// is why [`Self::covers`] is its own function rather than
/// [`AgingRow::covers`]: the Aging Roll table opens at exactly one end, and the
/// loader's integrity gates depend on that.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16624-16632.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrisisRow {
    /// Slug-style id, e.g. `crisis.minor_illness`. Its display text lives in
    /// `rules/i18n`, keyed by this id.
    pub id: Id,
    /// Lowest total the row covers (inclusive). `None` for the row open below
    /// ("8 or less", `:16626`), which has no lower bound at all rather than a
    /// very small one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i32>,
    /// Highest total the row covers (inclusive). `None` for the row open above
    /// ("19+", `:16632`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    /// What the row costs the character.
    pub outcome: CrisisOutcome,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl CrisisRow {
    /// Whether `total` lands on this row: the band is inclusive on both ends, and
    /// an absent bound is an open end — below for [`Self::min`] (`:16626`), above
    /// for [`Self::max`] (`:16632`).
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:16624-16632.
    fn covers(&self, total: i32) -> bool {
        self.min.is_none_or(|min| min <= total) && self.max.is_none_or(|max| total <= max)
    }
}

/// What a [`CrisisRow`] does to the character.
///
/// A tagged enum for the same reason [`AgingRowEffect`] is one: an outcome the
/// engine cannot read must fail at load rather than be silently ignored.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16624-16632.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CrisisOutcome {
    /// "Bedridden for a week" (`:16626`) and "Bedridden for a month." (`:16627`)
    /// — no roll, no spell, nothing but time. How long is display text in
    /// `rules/i18n`, keyed by the row's id, not a mechanic.
    Bedridden,
    /// An illness the character must survive: "Stamina stress roll against an
    /// Ease Factor of 3 or CrCo20 to survive" (`:16628`) and its four heavier
    /// siblings (`:16629-16632`).
    Illness {
        /// How bad it is, which is what fixes the required spell level: "The
        /// level of spell required depends on the severity of the crisis, as
        /// noted on the table." (`:16638`)
        severity: CrisisSeverity,
        /// The Ease Factor of the Stamina stress roll. `None` for the Terminal
        /// row, which offers no roll at all — "**Terminal illness**. CrCo40
        /// required to survive." (`:16632`) — rather than an unbeatable one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease_factor: Option<i32>,
        /// The level of the Momentary Creo Corpus Ritual that resolves the
        /// crisis (`:16638`), e.g. 20 for CrCo20.
        ritual_level: u32,
    },
}

/// How bad an [`CrisisOutcome::Illness`] is, ascending.
///
/// Ordered, and **declaration order is the ladder**: "The level of spell
/// required depends on the severity of the crisis, as noted on the table."
/// (`:16638`) The table's five illness rows climb together — 15/EF 3/CrCo20 up
/// to 19+/no roll/CrCo40 (`:16628-16632`) — so severity is a rank, not a label,
/// and comparing two of them is a rules operation the source licenses.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16628-16632, :16638.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrisisSeverity {
    /// "**Minor illness**. Stamina stress roll against an Ease Factor of 3 or
    /// CrCo20 to survive." (`:16628`)
    Minor,
    /// "**Serious illness**. … Ease Factor of 6 or CrCo25 to survive."
    /// (`:16629`)
    Serious,
    /// "**Major illness**. … Ease Factor of 9 or CrCo30 to survive." (`:16630`)
    Major,
    /// "**Critical illness**. … Ease Factor of 12 or CrCo35 to survive"
    /// (`:16631`)
    Critical,
    /// "**Terminal illness**. CrCo40 required to survive." (`:16632`)
    Terminal,
}

/// The die the crisis total is rolled on: "Roll a ten-sided die. Each number
/// counts for its value, except that a zero counts as ten." (`:474`) — so the
/// Simple Die's range is 1 to 10, which this carries as data rather than as a
/// literal in the engine.
///
/// The engine never rolls it (see the module docs); the bounds are here so a UI
/// can offer the legal results and the loader can reject a nonsensical range.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:474, :16621.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrisisDie {
    /// Lowest result the die can show (inclusive).
    pub min: i32,
    /// Highest result the die can show (inclusive).
    pub max: i32,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The doctor who may attend a crisis: "An Int + Medicine roll against an Ease
/// Factor of 6 allows the character to add the attendant's Medicine score to the
/// roll to survive the crisis. Only one doctor may usefully attend a patient, and
/// if the doctor botches the character must subtract 3 from the survival roll."
/// (`:16634`)
///
/// The Ability and Characteristic are data rather than hard-coded ids, so the
/// rule stays a property of the ruleset.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16634.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrisisAttendant {
    /// The Ability rolled and then added on a success — `ability.medicine`.
    pub ability: Id,
    /// The Characteristic added to the attendant's roll — Int.
    pub characteristic: Characteristic,
    /// The Ease Factor the attendant's roll must beat: 6.
    pub ease_factor: i32,
    /// What a botched attendance costs the patient, stored with its sign and
    /// **added** to the survival roll: -3, because "if the doctor botches the
    /// character must subtract 3 from the survival roll" (`:16634`).
    ///
    /// Signed-and-added, not a positive magnitude to subtract, because that is
    /// the convention every other aging modifier already follows — the aging
    /// roll's trait modifiers are added with their stored sign, and only the two
    /// terms `:16567-16569` *names* as subtracted are subtracted. One convention
    /// for one file; a second would invert something eventually.
    pub botch_penalty: i32,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effective::{
        characteristic_aging_drops, decrepitude_score, effective_characteristic_after_aging,
    };
    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::types::{
        AgingLogEntry, Entity, EntityKind, Familiar, LongevityRitual, LongevitySource, RulesetRef,
        Selection,
    };
    use pretty_assertions::assert_eq;

    /// The two tables of `## Aging` in miniature: a Living Conditions row of each
    /// kind (Core Rules.md:16581-16594) and one row of every outcome shape the
    /// Aging Roll table has (`:16597-16611`), plus the longevity clamp of
    /// `:16575`.
    const AGING: &str = r#"{
      "start_age": 35,
      "age_divisor": 10,
      "apparent_age_increase_min": 3,
      "longevity_clamp": { "max_total": 9, "until_age": 35 },
      "living_conditions": [
        { "id": "living_condition.average_peasant", "modifier": 0 },
        { "id": "living_condition.leper", "modifier": -2, "cumulative": true,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [16592, 16592] } }
      ],
      "outcomes": [
        { "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 },
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [16601, 16601] } },
        { "min": 13, "max": 13, "effect": { "type": "next_decrepitude_level_and_crisis" } },
        { "min": 18, "max": 18,
          "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["str", "sta"] } },
        { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }
      ]
    }"#;

    #[test]
    fn aging_rules_parse_the_two_tables_and_reject_an_unknown_effect_kind() {
        let rules: AgingRules =
            serde_json::from_str(AGING).expect("the shipped aging shape parses");

        assert_eq!(rules.start_age, 35);
        assert_eq!(rules.age_divisor, 10);
        assert_eq!(rules.apparent_age_increase_min, 3);
        let clamp = rules.longevity_clamp.clone().expect("the :16575 clamp");
        assert_eq!(clamp.max_total, 9);
        assert_eq!(clamp.until_age, 35);
        // An aging block carrying no crisis table still parses, unchanged.
        assert!(rules.crisis.is_none());
        assert!(rules.frail_decrepitude_score.is_none());
        assert!(rules.fatal_decrepitude_score.is_none());

        // A plain condition and a cumulative one, the asterisk of `:16594` being
        // the only thing that tells them apart.
        let conditions: Vec<(&str, i8, bool)> = rules
            .living_conditions
            .iter()
            .map(|row| (row.id.as_str(), row.modifier, row.cumulative))
            .collect();
        assert_eq!(
            conditions,
            vec![
                ("living_condition.average_peasant", 0, false),
                ("living_condition.leper", -2, true),
            ]
        );
        assert!(rules.living_conditions[0].source.is_none());
        let source = rules.living_conditions[1]
            .source
            .clone()
            .expect("provenance");
        assert_eq!(
            source.file,
            "Ars Magica - Definitive Edition (Core Rules).md"
        );
        assert_eq!((source.lines.start, source.lines.end), (16592, 16592));

        // Every outcome shape the table has: a banded range, an open-ended top
        // row, and all three effect kinds.
        let bands: Vec<(i32, Option<i32>)> = rules
            .outcomes
            .iter()
            .map(|row| (row.min, row.max))
            .collect();
        assert_eq!(
            bands,
            vec![(10, Some(12)), (13, Some(13)), (18, Some(18)), (22, None)]
        );
        let effects: Vec<&AgingRowEffect> = rules.outcomes.iter().map(|row| &row.effect).collect();
        assert_eq!(
            effects,
            vec![
                &AgingRowEffect::AnyCharacteristic { points: 1 },
                &AgingRowEffect::NextDecrepitudeLevelAndCrisis,
                &AgingRowEffect::NamedCharacteristics {
                    points: 1,
                    characteristics: vec![Characteristic::Str, Characteristic::Sta],
                },
                &AgingRowEffect::NextDecrepitudeLevelAndCrisis,
            ]
        );

        // Canonical JSON: a defaulted `cumulative`, an absent `max` and an absent
        // `source` all stay omitted, and re-reading gives back the same rules.
        let json = serde_json::to_string(&rules).expect("aging rules serialize");
        assert!(
            !json.contains("\"cumulative\":false"),
            "a defaulted flag must be skipped: {json}"
        );
        assert_eq!(
            json.matches("\"cumulative\":true").count(),
            1,
            "only the asterisked condition carries the flag: {json}"
        );
        assert_eq!(
            json.matches("\"max\"").count(),
            3,
            "the open-ended top row omits its bound: {json}"
        );
        assert_eq!(
            json.matches("\"source\"").count(),
            2,
            "rows without provenance omit the key: {json}"
        );

        let back: AgingRules = serde_json::from_str(&json).expect("the round trip re-reads");
        assert_eq!(back, rules);
        assert_eq!(serde_json::to_string(&back).expect("stable"), json);
    }

    #[test]
    fn an_unknown_effect_kind_is_rejected() {
        let newt = serde_json::from_str::<AgingRow>(
            r#"{ "min": 13, "effect": { "type": "turns_you_into_a_newt" } }"#,
        );
        assert!(
            newt.is_err(),
            "an effect kind the engine cannot apply must fail to load, not be ignored"
        );
    }

    /// The Crisis Table of `:16624-16632` in miniature — its two open ends, one
    /// `bedridden` row and one `illness` row of each shape (an Ease Factor, and
    /// the Terminal row that has none, `:16632`) — plus the Simple Die of `:474`
    /// and the attending doctor of `:16634`. A sibling of [`AGING`] rather than an
    /// extension of it: [`AGING`] is what an aging block with **no** crisis key
    /// looks like, and that case must keep parsing untouched.
    const AGING_WITH_CRISIS: &str = r#"{
      "start_age": 35,
      "age_divisor": 10,
      "apparent_age_increase_min": 3,
      "frail_decrepitude_score": 4,
      "fatal_decrepitude_score": 5,
      "crisis": {
        "die": { "min": 1, "max": 10,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [474, 474] } },
        "attendant": { "ability": "ability.medicine", "characteristic": "int",
          "ease_factor": 6, "botch_penalty": 3,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [16634, 16634] } },
        "rows": [
          { "id": "crisis.bedridden_week", "max": 8, "outcome": { "type": "bedridden" },
            "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [16626, 16626] } },
          { "id": "crisis.minor_illness", "min": 15, "max": 15,
            "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
          { "id": "crisis.terminal_illness", "min": 19,
            "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }
        ]
      }
    }"#;

    #[test]
    fn a_crisis_block_deserializes_its_rows_and_both_open_ends() {
        let rules: AgingRules =
            serde_json::from_str(AGING_WITH_CRISIS).expect("the crisis shape parses");

        assert_eq!(rules.frail_decrepitude_score, Some(4));
        assert_eq!(rules.fatal_decrepitude_score, Some(5));
        let crisis = rules.crisis.clone().expect("the :16624-16632 table");

        let ids: Vec<&str> = crisis.rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "crisis.bedridden_week",
                "crisis.minor_illness",
                "crisis.terminal_illness",
            ]
        );

        // "8 or less" (`:16626`) is open below and "19+" (`:16632`) open above —
        // the Crisis Table, unlike the Aging Roll table, has *both* ends open.
        let below = &crisis.rows[0];
        assert_eq!((below.min, below.max), (None, Some(8)));
        assert!(below.covers(i32::MIN));
        assert!(below.covers(8));
        assert!(!below.covers(9));

        let interior = &crisis.rows[1];
        assert!(!interior.covers(14));
        assert!(interior.covers(15));
        assert!(!interior.covers(16));

        let above = &crisis.rows[2];
        assert_eq!((above.min, above.max), (Some(19), None));
        assert!(!above.covers(18));
        assert!(above.covers(19));
        assert!(above.covers(i32::MAX));

        // Bedridden costs nothing but time; the illnesses carry a severity, the
        // Ease Factor of the Stamina roll, and the Creo Corpus level.
        assert_eq!(below.outcome, CrisisOutcome::Bedridden);
        assert_eq!(
            interior.outcome,
            CrisisOutcome::Illness {
                severity: CrisisSeverity::Minor,
                ease_factor: Some(3),
                ritual_level: 20,
            }
        );
        // "Terminal illness. CrCo40 required to survive." (`:16632`) — no Stamina
        // roll is offered at all, so the Ease Factor is absent, not zero.
        assert_eq!(
            above.outcome,
            CrisisOutcome::Illness {
                severity: CrisisSeverity::Terminal,
                ease_factor: None,
                ritual_level: 40,
            }
        );

        let die = crisis.die.clone().expect("the :474 Simple Die");
        assert_eq!((die.min, die.max), (1, 10));
        let attendant = crisis.attendant.clone().expect("the :16634 doctor");
        assert_eq!(attendant.ability.as_str(), "ability.medicine");
        assert_eq!(attendant.characteristic, Characteristic::Int);
        assert_eq!(attendant.ease_factor, 6);
        assert_eq!(attendant.botch_penalty, 3);
        let source = attendant.source.clone().expect("provenance");
        assert_eq!(
            source.file,
            "Ars Magica - Definitive Edition (Core Rules).md"
        );
        assert_eq!((source.lines.start, source.lines.end), (16634, 16634));

        // Canonical JSON: every absent option stays omitted, and re-reading gives
        // back the same rules.
        let json = serde_json::to_string(&rules).expect("crisis rules serialize");
        assert!(
            json.contains(r#"{"id":"crisis.bedridden_week","max":8"#),
            "the row open below omits its absent lower bound: {json}"
        );
        assert!(
            json.contains(r#"{"id":"crisis.terminal_illness","min":19,"outcome""#),
            "the row open above omits its absent upper bound: {json}"
        );
        assert_eq!(
            json.matches("\"min\"").count(),
            3,
            "the die's bound and the two rows with a lower bound: {json}"
        );
        assert_eq!(
            json.matches("\"max\"").count(),
            3,
            "the die's bound and the two rows with an upper bound: {json}"
        );
        assert_eq!(
            json.matches("\"ease_factor\"").count(),
            2,
            "the Terminal row offers no Stamina roll: {json}"
        );
        let back: AgingRules = serde_json::from_str(&json).expect("the round trip re-reads");
        assert_eq!(back, rules);
        assert_eq!(serde_json::to_string(&back).expect("stable"), json);
    }

    #[test]
    fn an_unknown_crisis_outcome_kind_is_rejected() {
        let newt = serde_json::from_str::<CrisisRow>(
            r#"{ "id": "crisis.newt", "min": 15, "outcome": { "type": "turns_you_into_a_newt" } }"#,
        );
        assert!(
            newt.is_err(),
            "a crisis outcome the engine cannot read must fail to load, not be ignored"
        );
    }

    #[test]
    fn crisis_severity_ascends_in_declaration_order() {
        assert!(CrisisSeverity::Minor < CrisisSeverity::Serious);
        assert!(CrisisSeverity::Serious < CrisisSeverity::Major);
        assert!(CrisisSeverity::Major < CrisisSeverity::Critical);
        assert!(CrisisSeverity::Critical < CrisisSeverity::Terminal);
    }

    /// A ruleset carrying a loadable aging block. [`AGING`] itself will not do:
    /// its miniature table leaves a gap between 13 and 18, which the loader's
    /// tiling gate rejects, so the schedule fixtures ship a contiguous one.
    ///
    /// The Living Conditions rows are the baseline (`:16587`), a positive
    /// alternative (`:16583`) and two asterisked ones that stack (`:16590`,
    /// `:16592`); the point items are the three shipped carriers of a
    /// `living_conditions` aging modifier — plus one Personality Flaw, which any
    /// ruleset shipping a V/F catalogue at all must carry
    /// (`validate_engine_required_categories`), the shipped `aging_roll` carrier
    /// Faerie Blood (`:3801`), and one **synthetic** carrier of a
    /// `longevity_bonus` modifier, a kind no shipped item uses today.
    ///
    /// The eleven Aging Roll rows are the book's own (`:16601-16611`), because
    /// [`resolve_outcome`] has to be witnessed against every row shape the table
    /// actually has. The advancement curve is the shipped one's first three rows
    /// (5 / 15 / 30), deliberately **short**: a table that tops out is what lets a
    /// test reach the score the curve cannot price.
    fn scheduled_ruleset() -> Ruleset {
        let aging = r#"{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          "longevity_clamp": { "max_total": 9, "until_age": 35 },
          "living_conditions": [
            { "id": "living_condition.average_peasant", "modifier": 0 },
            { "id": "living_condition.leper", "modifier": -2, "cumulative": true },
            { "id": "living_condition.wealthy_or_healthy_location", "modifier": 2 },
            { "id": "living_condition.work_in_a_mine", "modifier": -1, "cumulative": true }
          ],
          "outcomes": [
            { "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 } },
            { "min": 13, "max": 13, "effect": { "type": "next_decrepitude_level_and_crisis" } },
            { "min": 14, "max": 14,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["qik"] } },
            { "min": 15, "max": 15,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["sta"] } },
            { "min": 16, "max": 16,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["per"] } },
            { "min": 17, "max": 17,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["pre"] } },
            { "min": 18, "max": 18,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["str", "sta"] } },
            { "min": 19, "max": 19,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["dex", "qik"] } },
            { "min": 20, "max": 20,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["com", "pre"] } },
            { "min": 21, "max": 21,
              "effect": { "type": "named_characteristics", "points": 1, "characteristics": ["int", "per"] } },
            { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }
          ]
        }"#;
        let abilities = r#"{
          "abilities": [],
          "advancement": [
            { "score": 1, "total_xp": 5 },
            { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }
          ]
        }"#;
        let items = r#"[
          { "id": "flaw.driven", "kind": "flaw", "magnitude": "minor",
            "category": "personality", "classification": "narrative" },
          { "id": "flaw.poor_living_conditions", "kind": "flaw", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "living_conditions", "amount": -1 }] },
          { "id": "virtue.mild_aging", "kind": "virtue", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "living_conditions", "amount": 1 }] },
          { "id": "virtue.unaging", "kind": "virtue", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "no_aging", "amount": 0 },
                        { "type": "aging_mod", "kind": "no_apparent_aging", "amount": 0 }] },
          { "id": "flaw.bound_to_role_role", "kind": "flaw", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "no_aging", "amount": 0 }] },
          { "id": "virtue.bee_king", "kind": "virtue", "magnitude": "major",
            "category": "supernatural", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "no_apparent_aging", "amount": 0 }] },
          { "id": "virtue.faerie_blood", "kind": "virtue", "magnitude": "minor",
            "category": "supernatural", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "aging_roll", "amount": -1 }] },
          { "id": "virtue.synthetic_longevity_bonus", "kind": "virtue", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "longevity_bonus", "amount": 2 }] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: "[]",
            abilities: Some(abilities),
            aging: Some(aging),
            ..RulesetSources::default()
        })
        .expect("the aging fixture loads")
    }

    /// A ruleset whose aging block ships a **Crisis Table** as well as the two
    /// aging tables. [`scheduled_ruleset`] deliberately ships none — an aging
    /// block written before the crisis existed must keep loading — so a crisis
    /// total against it is `None`, which is its own test below.
    ///
    /// Both tables are the smallest that survive the loader's gates. The Crisis
    /// Table tiles the integers contiguously with the open-below row (`:16626`)
    /// first and the open-above row (`:16632`) last; the Aging Roll table carries
    /// "10-12" (`:16601`) for a year that costs one point and an open-ended
    /// "13+" standing in for `:16602`/`:16611`, so a test can drive a year to a
    /// Crisis on demand.
    ///
    /// The advancement curve is the shipped one's first five rows (5 / 15 / 30 /
    /// 50 / 75) rather than [`scheduled_ruleset`]'s deliberately short three: the
    /// crisis total has to be witnessed while the Decrepitude score is still
    /// climbing, not while it sits at a curve that has run out.
    ///
    /// It ships **no attendant**, so the `:16634` doctor's absence is testable;
    /// the shipped table carries one, and `data_integrity.rs` tests that half.
    /// Its three items are the two sides of the `:16636` wall — an aging-ROLL
    /// modifier, a LIVING CONDITIONS one, and Mild Aging, which carries one of
    /// each kind plus the crisis-survival grant.
    fn crisis_ruleset() -> Ruleset {
        let aging = r#"{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          "frail_decrepitude_score": 4,
          "fatal_decrepitude_score": 5,
          "outcomes": [
            { "min": 10, "max": 12, "effect": { "type": "any_characteristic", "points": 1 } },
            { "min": 13, "effect": { "type": "next_decrepitude_level_and_crisis" } }
          ],
          "crisis": {
            "die": { "min": 1, "max": 10 },
            "rows": [
              { "id": "crisis.bedridden_week", "max": 8, "outcome": { "type": "bedridden" } },
              { "id": "crisis.bedridden_month", "min": 9, "max": 14,
                "outcome": { "type": "bedridden" } },
              { "id": "crisis.minor_illness", "min": 15, "max": 15,
                "outcome": { "type": "illness", "severity": "minor", "ease_factor": 3, "ritual_level": 20 } },
              { "id": "crisis.terminal_illness", "min": 16,
                "outcome": { "type": "illness", "severity": "terminal", "ritual_level": 40 } }
            ]
          }
        }"#;
        let abilities = r#"{
          "abilities": [],
          "advancement": [
            { "score": 1, "total_xp": 5 },
            { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 },
            { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ]
        }"#;
        let items = r#"[
          { "id": "flaw.driven", "kind": "flaw", "magnitude": "minor",
            "category": "personality", "classification": "narrative" },
          { "id": "virtue.faerie_blood", "kind": "virtue", "magnitude": "minor",
            "category": "supernatural", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "aging_roll", "amount": -1 }] },
          { "id": "flaw.poor_living_conditions", "kind": "flaw", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "living_conditions", "amount": -1 }] },
          { "id": "virtue.mild_aging", "kind": "virtue", "magnitude": "minor",
            "category": "general", "classification": "in_play_effect",
            "effects": [{ "type": "aging_mod", "kind": "living_conditions", "amount": 1 },
                        { "type": "aging_mod", "kind": "crisis_survival", "amount": 3 }] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: "[]",
            abilities: Some(abilities),
            aging: Some(aging),
            ..RulesetSources::default()
        })
        .expect("the crisis fixture loads")
    }

    /// The [`CrisisOutcome`] of one named row of a fixture's Crisis Table — what
    /// a look-up on the table would hand [`crisis_survival`], without a total to
    /// get there.
    fn crisis_outcome(ruleset: &Ruleset, id: &str) -> CrisisOutcome {
        ruleset
            .aging()
            .expect("the fixture ships aging rules")
            .crisis
            .as_ref()
            .expect("the fixture ships a Crisis Table")
            .rows
            .iter()
            .find(|row| row.id.as_str() == id)
            .unwrap_or_else(|| panic!("the fixture ships the row '{id}'"))
            .outcome
            .clone()
    }

    /// A character of `age`, born in `birth_year` when one is given.
    fn character(age: Option<u32>, birth_year: Option<i32>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = age;
        entity.birth_year = birth_year;
        entity
    }

    /// The ages of an entity's schedule, which is what most of these assert on.
    fn ages(entity: &Entity, ruleset: &Ruleset) -> Vec<u32> {
        aging_schedule(entity, ruleset)
            .into_iter()
            .map(|year| year.age)
            .collect()
    }

    /// The one deliberate off-by-one in the engine, and the three sentences that
    /// settle it.
    ///
    /// `:16565` — "Characters begin aging in the Winter **after** they turn 35.
    /// Every year, a character must roll on the aging table." The Winter *after*
    /// the 35th birthday falls in the character's 36th year, so 35 is the last
    /// year owing nothing and 36 is the first owing a roll.
    ///
    /// `:2496` reads against this and is disposed of, not ignored: "you should
    /// also make aging rolls for the character each year **from the age of 35**".
    /// That is advice inside the worked magus-advancement example; `:2232` is the
    /// creation-time rule proper — "a character **over** the age of 35 must make
    /// aging rolls … before the game begins" — and it agrees with `:16565`. Over
    /// 35, from the Winter after 35: the first owed roll is at 36.
    #[test]
    fn aging_rolls_are_owed_from_the_year_after_thirty_five() {
        let ruleset = scheduled_ruleset();
        let rules = ruleset.aging().expect("the fixture ships aging rules");
        assert_eq!(rules.start_age, 35);
        assert_eq!(rules.first_roll_age(), 36);

        // A 38-year-old owes the three years he has lived past the threshold.
        assert_eq!(ages(&character(Some(38), None), &ruleset), vec![36, 37, 38]);

        // At the threshold itself nothing is owed — he has turned 35, but the
        // Winter after has not come.
        assert!(ages(&character(Some(35), None), &ruleset).is_empty());
        assert!(ages(&character(Some(20), None), &ruleset).is_empty());

        // And one year past it, exactly one roll.
        assert_eq!(ages(&character(Some(36), None), &ruleset), vec![36]);
    }

    /// [`AgingLogEntry`](crate::types::AgingLogEntry) records a **calendar** year,
    /// while the schedule is driven by the character's age — so each entry carries
    /// both, and the pairing is the schedule's job rather than every caller's.
    #[test]
    fn the_aging_schedule_pairs_each_age_with_its_calendar_year() {
        let ruleset = scheduled_ruleset();

        let born = character(Some(38), Some(1160));
        assert_eq!(
            aging_schedule(&born, &ruleset),
            vec![
                AgingYear {
                    age: 36,
                    year: Some(1196),
                    recorded: false,
                },
                AgingYear {
                    age: 37,
                    year: Some(1197),
                    recorded: false,
                },
                AgingYear {
                    age: 38,
                    year: Some(1198),
                    recorded: false,
                },
            ]
        );

        // Birth year is optional flavor, so a character without one still owes the
        // same rolls — they just have no calendar year to sit in.
        let undated = character(Some(38), None);
        let schedule = aging_schedule(&undated, &ruleset);
        assert_eq!(schedule.len(), 3);
        assert!(
            schedule.iter().all(|entry| entry.year.is_none()),
            "no birth year means no calendar year: {schedule:?}"
        );
    }

    /// Two ways the question does not arise at all: a character whose age was
    /// never entered, and a ruleset that ships no aging tables (which stands the
    /// whole subsystem down rather than letting the engine invent a threshold).
    #[test]
    fn a_character_without_an_age_or_aging_rules_owes_no_schedule() {
        let ruleset = scheduled_ruleset();
        assert!(aging_schedule(&character(None, Some(1160)), &ruleset).is_empty());

        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(without.aging().is_none());
        assert!(aging_schedule(&character(Some(80), Some(1160)), &without).is_empty());
    }

    /// A character of `age` living under the named conditions.
    fn living_under(conditions: &[&str]) -> Entity {
        let mut entity = character(Some(40), None);
        entity.living_conditions = conditions.iter().map(|id| Id::new(*id)).collect();
        entity
    }

    /// The Living Conditions table (`:16581-16594`): the asterisked rows "are
    /// cumulative with each other" (`:16594`), so a leper working in a mine holds
    /// both and their modifiers add. An empty set is not an incomplete entry — it
    /// is the table's own baseline, "Average peasant 0" (`:16587`).
    #[test]
    fn the_living_conditions_modifier_sums_the_chosen_rows() {
        let ruleset = scheduled_ruleset();

        let stacked = living_conditions_modifier(
            &living_under(&["living_condition.leper", "living_condition.work_in_a_mine"]),
            &ruleset,
        );
        assert_eq!(
            stacked.rows,
            vec![
                Id::new("living_condition.leper"),
                Id::new("living_condition.work_in_a_mine"),
            ]
        );
        assert_eq!(stacked.from_table, -3);
        assert_eq!(stacked.from_traits, 0);
        assert_eq!(stacked.total, -3);

        // A single row resolves to its own modifier, sign intact: `:16571` says a
        // high modifier means a longer life, and the total *subtracts* it, so the
        // book's own signs are what this function returns.
        let wealthy = living_conditions_modifier(
            &living_under(&["living_condition.wealthy_or_healthy_location"]),
            &ruleset,
        );
        assert_eq!(wealthy.from_table, 2);
        assert_eq!(wealthy.total, 2);

        let baseline = living_conditions_modifier(&living_under(&[]), &ruleset);
        assert!(baseline.rows.is_empty());
        assert_eq!(baseline.total, 0);
    }

    /// An id the table does not know contributes nothing and is skipped here; a
    /// later step reports it as a validation finding, which is the project's one
    /// evaluation path — compute, then report, never panic mid-computation.
    #[test]
    fn an_unknown_living_condition_contributes_nothing() {
        let ruleset = scheduled_ruleset();
        let modifier = living_conditions_modifier(
            &living_under(&[
                "living_condition.leper",
                "living_condition.marooned_on_the_moon",
            ]),
            &ruleset,
        );

        assert_eq!(modifier.rows, vec![Id::new("living_condition.leper")]);
        assert_eq!(modifier.from_table, -2);
        assert_eq!(modifier.total, -2);
    }

    /// Mild Aging gives "a +1 bonus to the Living Conditions Modifier" (`:4530`)
    /// and Poor Living Conditions "an additional -1 Living Conditions Modifier …
    /// cumulative with the character's base" (`:6620`) — so a Virtue/Flaw modifier
    /// joins the table rows in the total without disturbing `from_table`.
    #[test]
    fn virtue_living_condition_modifiers_join_the_table_rows() {
        let ruleset = scheduled_ruleset();

        let mut entity = living_under(&["living_condition.work_in_a_mine"]);
        entity.selections = vec![
            Selection::new(Id::new("virtue.mild_aging")),
            Selection::new(Id::new("virtue.unaging")),
        ];
        let mild = living_conditions_modifier(&entity, &ruleset);
        assert_eq!(mild.rows, vec![Id::new("living_condition.work_in_a_mine")]);
        assert_eq!(mild.from_table, -1, "the table rows are untouched");
        assert_eq!(
            mild.from_traits, 1,
            "only the living_conditions kind counts"
        );
        assert_eq!(mild.total, 0);

        // And the Flaw pulls the other way, with no table row at all.
        let mut poor = living_under(&[]);
        poor.selections = vec![Selection::new(Id::new("flaw.poor_living_conditions"))];
        let poor = living_conditions_modifier(&poor, &ruleset);
        assert_eq!(poor.from_table, 0);
        assert_eq!(poor.from_traits, -1);
        assert_eq!(poor.total, -1);
    }

    /// "age/10 (round up)" (`:16567`) — so the term steps up on the first year of
    /// each decade, not the last: 30 still scores 3, and 31 already scores 4.
    #[test]
    fn the_age_modifier_rounds_the_decade_up() {
        let ruleset = scheduled_ruleset();
        let rules = ruleset.aging().expect("the fixture ships aging rules");

        assert_eq!(rules.age_modifier(30), 3);
        assert_eq!(rules.age_modifier(31), 4);
        assert_eq!(rules.age_modifier(40), 4);
        assert_eq!(rules.age_modifier(41), 5);
    }

    /// A character with a Longevity Ritual of the given entered bonus.
    fn with_ritual(entity: &mut Entity, bonus: Option<i8>) {
        entity.longevity_ritual = Some(LongevityRitual {
            source: LongevitySource::External,
            bonus,
            focus: String::new(),
        });
    }

    /// The four terms of the formula and, crucially, their **signs**:
    ///
    /// > **AGING TOTAL: Stress die (no botch) + age/10 (round up)**
    /// > **\- Living Conditions modifier**
    /// > **\- Longevity Ritual modifier** (`:16567-16569`)
    ///
    /// Every term is given a different magnitude here, so a swapped pair cannot
    /// coincidentally produce the right sum.
    #[test]
    fn the_aging_total_adds_the_age_step_and_subtracts_the_two_modifiers() {
        let ruleset = scheduled_ruleset();
        let mut entity =
            living_under(&["living_condition.leper", "living_condition.work_in_a_mine"]);
        entity.selections = vec![Selection::new(Id::new("virtue.faerie_blood"))];
        with_ritual(&mut entity, Some(5));

        let total = aging_total(&entity, &ruleset, 60, 7).expect("the fixture ships aging rules");

        assert_eq!(total.age, 60);
        assert_eq!(total.die, 7);
        assert_eq!(total.age_modifier, 6, "ceil(60/10), ADDED");
        assert_eq!(total.living_conditions.total, -3, "the book's own sign");
        assert_eq!(total.longevity_bonus, 5, "the entered ritual bonus");
        assert_eq!(total.trait_modifier, -1, "Faerie Blood's -1 aging roll");
        // 7 + 6 - (-3) - 5 + (-1)
        assert_eq!(total.uncapped_total, 10);
        assert_eq!(total.total, 10);
        assert!(!total.capped_by_longevity);

        // A ruleset with no aging rules has no total to compute.
        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(aging_total(&entity, &without, 60, 7).is_none());
    }

    /// The sign trap, stated as an assertion so it cannot be "simplified" away.
    ///
    /// Mild Aging ships `+1` and Poor Living Conditions `-1` — both in the book's
    /// own sign (`:4530`, `:6620`) — and the AGING TOTAL *subtracts* the Living
    /// Conditions modifier. So the Virtue must LOWER the total and the Flaw must
    /// RAISE it, from the very same die.
    #[test]
    fn mild_aging_and_poor_living_conditions_move_the_total_in_opposite_directions() {
        let ruleset = scheduled_ruleset();
        let total_of = |item: Option<&str>| {
            let mut entity = living_under(&[]);
            entity.selections = item
                .map(|id| Selection::new(Id::new(id)))
                .into_iter()
                .collect();
            aging_total(&entity, &ruleset, 40, 6)
                .expect("the fixture ships aging rules")
                .total
        };

        // 6 + ceil(40/10), no modifiers at all.
        assert_eq!(total_of(None), 10);
        assert_eq!(
            total_of(Some("virtue.mild_aging")),
            9,
            "+1 conditions LOWERS"
        );
        assert_eq!(
            total_of(Some("flaw.poor_living_conditions")),
            11,
            "-1 conditions RAISES"
        );
    }

    /// A `longevity_bonus` modifier moves the ritual term — but only for a
    /// character who actually has a ritual, since a modifier to a ritual bonus is
    /// meaningless without one. Synthetic: no shipped item carries this kind.
    #[test]
    fn a_longevity_bonus_modifier_moves_the_ritual_term_only_with_a_ritual() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        entity.selections = vec![Selection::new(Id::new("virtue.synthetic_longevity_bonus"))];

        let without = aging_total(&entity, &ruleset, 40, 6).expect("aging rules");
        assert_eq!(without.longevity_bonus, 0, "no ritual, nothing to modify");
        assert_eq!(without.total, 10);

        with_ritual(&mut entity, Some(4));
        let with = aging_total(&entity, &ruleset, 40, 6).expect("aging rules");
        assert_eq!(
            with.longevity_bonus, 6,
            "the entered 4 plus the modifier's 2"
        );
        assert_eq!(with.total, 4);

        // An unentered bonus is not a claimed 0 — but it contributes 0 here, and
        // the modifier still applies, because the ritual exists.
        with_ritual(&mut entity, None);
        let unentered = aging_total(&entity, &ruleset, 40, 6).expect("aging rules");
        assert_eq!(unentered.longevity_bonus, 2);
    }

    /// "The modifier to rolls depends on the character's **actual, not apparent**,
    /// age." (`:16577`) — so an apparent age far from the real one moves nothing.
    #[test]
    fn the_aging_total_reads_the_actual_age_not_the_apparent_age() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        entity.age = Some(60);
        entity.apparent_age = Some(20);

        let total = aging_total(&entity, &ruleset, 60, 6).expect("aging rules");
        assert_eq!(total.age_modifier, 6, "ceil(60/10), not ceil(20/10)");
        assert_eq!(total.total, 12);

        // And the apparent age is not consulted even when it is the higher figure.
        entity.apparent_age = Some(90);
        assert_eq!(
            aging_total(&entity, &ruleset, 60, 6).expect("aging rules"),
            total
        );
    }

    /// The `:16575` clamp: a ritual-holder "treats all rolls of 10 or more as
    /// rolls of 9 until he reaches the age of 35", so he "is at no risk of
    /// actually aging before any other characters".
    ///
    /// It is a CEILING on the TOTAL, never a floor and never a cap on the die —
    /// see [`aging_total`]'s doc for why the die reading cannot be right.
    #[test]
    fn a_longevity_ritual_caps_the_total_at_nine_before_thirty_five() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        with_ritual(&mut entity, Some(1));

        // 9 + ceil(34/10) - 1 = 12, which the clamp brings back to 9 — the last
        // total below the table's first aging-point row.
        let young = aging_total(&entity, &ruleset, 34, 9).expect("aging rules");
        assert_eq!(young.uncapped_total, 12);
        assert_eq!(young.total, 9);
        assert!(young.capped_by_longevity);

        // The one-year seam: `until_age` is 35, so at exactly 35 the same
        // character rolls unclamped — while a character *without* a ritual does
        // not roll at all until 36.
        let at_thirty_five = aging_total(&entity, &ruleset, 35, 9).expect("aging rules");
        assert_eq!(at_thirty_five.total, 12);
        assert!(!at_thirty_five.capped_by_longevity);

        // No ritual, no clamp, at any age.
        let mut unritualed = living_under(&[]);
        unritualed.longevity_ritual = None;
        let bare = aging_total(&unritualed, &ruleset, 34, 9).expect("aging rules");
        assert_eq!(bare.uncapped_total, 13);
        assert_eq!(bare.total, 13);
        assert!(!bare.capped_by_longevity);

        // A ceiling, never a floor: a low total is left exactly where it is.
        let mut wealthy = living_under(&["living_condition.wealthy_or_healthy_location"]);
        with_ritual(&mut wealthy, Some(1));
        let low = aging_total(&wealthy, &ruleset, 34, 1).expect("aging rules");
        assert_eq!(low.uncapped_total, 2, "1 + 4 - 2 - 1");
        assert_eq!(low.total, 2);
        assert!(!low.capped_by_longevity);
    }

    /// "2 or less — No apparent aging" / "3 or more — Apparent age increases by
    /// one year" (`:16599-16600`). Not two rows of the outcome table but one
    /// threshold asked of every total, and it fires long before any total costs a
    /// Characteristic anything.
    #[test]
    fn the_appearance_ages_from_three_and_not_from_two() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        let two = resolve_outcome(&entity, &ruleset, 2).expect("the fixture ships aging rules");
        assert_eq!(two.total, 2, "the resolved total is echoed back");
        assert!(
            !two.apparent_age_increases,
            "'2 or less: No apparent aging'"
        );
        assert!(two.awards.is_empty());
        assert!(!two.crisis);

        let three = resolve_outcome(&entity, &ruleset, 3).expect("aging rules");
        assert!(
            three.apparent_age_increases,
            "'3 or more: Apparent age increases by one year'"
        );
        assert!(
            three.awards.is_empty(),
            "the appearance ages well below the first row that costs Aging Points"
        );
        assert!(!three.crisis);
    }

    /// A character carrying that Virtue, and nothing else.
    fn carrying(item: &str) -> Entity {
        let mut entity = living_under(&[]);
        entity.selections = vec![Selection::new(Id::new(item))];
        entity
    }

    /// "Bee Kings do not appear to age after reaching maturity" (`:3488`) — and
    /// that is *all* it says. The appearance stops; the body does not. So the
    /// table's award is exactly the one anybody else takes at the same total, and
    /// the point it places still forces a Characteristic drop, because dropping is
    /// what `no_aging` (`:5189`) suppresses and a Bee King carries no `no_aging`.
    ///
    /// The shipped catalogue tagged this Virtue `no_aging` — the wrong one of the
    /// two facts, and the reason the tags had to come apart.
    #[test]
    fn a_bee_king_does_not_appear_to_age_but_still_loses_characteristics() {
        let ruleset = scheduled_ruleset();
        let ordinary =
            resolve_outcome(&living_under(&[]), &ruleset, 15).expect("the fixture ships a table");
        assert!(
            ordinary.apparent_age_increases,
            "15 ages anybody else's appearance"
        );

        let mut bee_king = carrying("virtue.bee_king");
        let outcome = resolve_outcome(&bee_king, &ruleset, 15).expect("aging rules");
        assert!(
            !outcome.apparent_age_increases,
            "'Bee Kings do not appear to age' (:3488)"
        );
        assert_eq!(
            outcome.awards, ordinary.awards,
            "the row's Aging Points are untouched — only the appearance is spared"
        );

        // And the point the row places still costs him the Characteristic: a Sta
        // of 0 drops on its first aging point (`:16579`).
        bee_king.aging_points.insert(Characteristic::Sta, 1);
        assert_eq!(
            characteristic_aging_drops(&bee_king, &ruleset),
            BTreeMap::from([(Characteristic::Sta, 1)]),
            "no_apparent_aging says nothing about the Characteristics"
        );
    }

    /// The two items diverge on exactly one axis, from the same total.
    ///
    /// "This Flaw also includes the effects of the Unaging Virtue, **but** the
    /// character's apparent age advances in line with their physical age"
    /// (`:5743`). That *but* is the proof the two facts are separable: Bound to
    /// (Role) keeps Unaging's Characteristic immunity and withholds `:5189`'s "You
    /// may choose your apparent age freely".
    #[test]
    fn bound_to_role_keeps_ageing_in_appearance_while_unaging_does_not() {
        let ruleset = scheduled_ruleset();

        let bound = resolve_outcome(&carrying("flaw.bound_to_role_role"), &ruleset, 15)
            .expect("aging rules");
        assert!(
            bound.apparent_age_increases,
            "'the character's apparent age advances in line with their physical age' (:5743)"
        );

        let unaging = resolve_outcome(&carrying("virtue.unaging"), &ruleset, 15).expect("rules");
        assert!(
            !unaging.apparent_age_increases,
            "'You may choose your apparent age freely' (:5189)"
        );

        assert_eq!(
            bound.awards, unaging.awards,
            "both carry no_aging, so they differ on the appearance alone"
        );
    }

    /// "In game terms, your aging points do not decrease your Characteristics,
    /// **only building up to give you Decrepitude points**" (`:5189`).
    ///
    /// Both halves are load-bearing, and they pull in opposite directions: the
    /// Characteristic is spared, and the very same points still accrue and still
    /// raise Decrepitude. So `aging_drops` returns nothing for a `no_aging` carrier
    /// while `decrepitude_points_total` / `decrepitude_score` are untouched — the
    /// character ages into Decrepitude exactly as fast as anyone else.
    ///
    /// The control is the same character without the Virtue, which guards the
    /// `:16613` worked examples: suppression must be the carrier's, not everyone's.
    #[test]
    fn unaging_accrues_decrepitude_without_dropping_a_characteristic() {
        let ruleset = scheduled_ruleset();
        // From a Stamina of 0, 21 aging points force six drops (`:16579`).
        let aged = |item: Option<&str>| {
            let mut entity = match item {
                Some(item) => carrying(item),
                None => living_under(&[]),
            };
            entity.characteristics.insert(Characteristic::Sta, 0);
            entity.aging_points.insert(Characteristic::Sta, 21);
            entity
        };

        let ordinary = aged(None);
        assert_eq!(
            characteristic_aging_drops(&ordinary, &ruleset),
            BTreeMap::from([(Characteristic::Sta, 6)]),
            "without the Virtue the points cost Characteristics as ever (:16579)"
        );

        let unaging = aged(Some("virtue.unaging"));
        assert!(
            characteristic_aging_drops(&unaging, &ruleset).is_empty(),
            "'your aging points do not decrease your Characteristics' (:5189)"
        );
        assert_eq!(
            effective_characteristic_after_aging(&unaging, &ruleset, Characteristic::Sta),
            0,
            "the Characteristic itself never moves"
        );

        // …and the points still build up into Decrepitude, at the same rate.
        assert_eq!(decrepitude_points_total(&unaging), 21);
        assert_eq!(
            decrepitude_score(&unaging, &ruleset),
            decrepitude_score(&ordinary, &ruleset),
            "'only building up to give you Decrepitude points' (:5189)"
        );
        assert!(decrepitude_score(&unaging, &ruleset) > 0);
    }

    /// Rows 14-21 name the Characteristics themselves (`:16603-16610`), one or
    /// two of them, each taking a point of its own. The book writes Presence as
    /// "Prs"; the enum variant is `Pre`.
    #[test]
    fn the_table_names_the_characteristics_for_fourteen_through_twenty_one() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);
        let named = |total: i32| -> Vec<Characteristic> {
            let outcome = resolve_outcome(&entity, &ruleset, total).expect("aging rules");
            assert!(
                outcome.apparent_age_increases,
                "total {total} is far above the apparent-aging threshold"
            );
            assert!(!outcome.crisis, "total {total} carries no Crisis");
            outcome
                .awards
                .iter()
                .map(|award| {
                    assert_eq!(
                        award.points,
                        Some(1),
                        "each named Characteristic takes one point, not a share of one"
                    );
                    match award.target {
                        AgingPointTarget::Named(characteristic) => characteristic,
                        ref other => {
                            panic!("total {total} should name a Characteristic: {other:?}")
                        }
                    }
                })
                .collect()
        };

        assert_eq!(named(14), vec![Characteristic::Qik]);
        assert_eq!(named(18), vec![Characteristic::Str, Characteristic::Sta]);
        assert_eq!(named(20), vec![Characteristic::Com, Characteristic::Pre]);
        assert_eq!(named(21), vec![Characteristic::Int, Characteristic::Per]);
    }

    /// "10–12 — 1 Aging Point in any Characteristic" (`:16601`), and "If an Aging
    /// Point 'in any Characteristic' is gained, the player may choose the
    /// Characteristic" (`:16615`) — so the engine names none of them.
    #[test]
    fn ten_through_twelve_leave_the_characteristic_to_the_player() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        for total in 10..=12 {
            let outcome = resolve_outcome(&entity, &ruleset, total).expect("aging rules");
            assert_eq!(
                outcome.awards,
                vec![AgingPointAward {
                    target: AgingPointTarget::PlayerChoice,
                    points: Some(1),
                }],
                "total {total} gives exactly one point, placed by the player"
            );
            assert!(outcome.apparent_age_increases, "total {total}");
            assert!(!outcome.crisis, "total {total}");
        }
    }

    /// "Gain sufficient Aging Points (in any Characteristics) to reach the next
    /// level in Decrepitude, and Crisis" (`:16602`, `:16611`).
    ///
    /// The count is derived, never authored: Decrepitude "increases as an
    /// Ability" off the accrued points (`:16617`), so the next level costs the
    /// advancement curve's price for the next score less what the character has
    /// already accrued. The expectation is computed from the table rather than
    /// written out, so a re-priced curve moves the test with it.
    #[test]
    fn thirteen_and_twenty_two_reach_the_next_decrepitude_level_and_flag_a_crisis() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        entity.aging_points.insert(Characteristic::Str, 3);
        assert_eq!(
            decrepitude_score(&entity, &ruleset),
            0,
            "three accrued points is short of Decrepitude 1"
        );

        let to_first_level = ruleset
            .advancement()
            .xp_for_score(1)
            .expect("the fixture's curve prices Decrepitude 1")
            - 3;
        let owed = vec![AgingPointAward {
            target: AgingPointTarget::NextDecrepitudeLevel,
            points: Some(to_first_level),
        }];

        // 13 (`:16602`), and the open-ended top row (`:16611`) whatever the total.
        for total in [13, 22, 30] {
            let outcome = resolve_outcome(&entity, &ruleset, total).expect("aging rules");
            assert_eq!(outcome.awards, owed, "total {total}");
            assert!(
                outcome.crisis,
                "total {total} sends the character to a Crisis"
            );
            assert!(outcome.apparent_age_increases, "total {total}");
        }

        // Every other row on the table is crisis-free.
        for total in (2..=21).filter(|total| *total != 13) {
            let outcome = resolve_outcome(&entity, &ruleset, total).expect("aging rules");
            assert!(!outcome.crisis, "total {total} carries no Crisis");
        }

        // A character the curve can no longer price: his Decrepitude is already
        // the table's top score, so "the next level" has no price at all. That is
        // reported as unpriceable, never silently costed at 0.
        let top = ruleset
            .advancement()
            .max_score()
            .expect("the fixture's curve prices something");
        let at_top = ruleset
            .advancement()
            .xp_for_score(top)
            .expect("the top score is priced");
        let mut frail = living_under(&[]);
        frail.aging_points.insert(
            Characteristic::Str,
            u8::try_from(at_top).expect("the fixture's top price fits a point count"),
        );
        assert_eq!(decrepitude_score(&frail, &ruleset), top);

        let outcome = resolve_outcome(&frail, &ruleset, 22).expect("aging rules");
        assert_eq!(
            outcome.awards,
            vec![AgingPointAward {
                target: AgingPointTarget::NextDecrepitudeLevel,
                points: None,
            }],
            "a level the curve cannot price is reported as unpriceable"
        );
        assert!(outcome.crisis);
    }

    /// Below "10–12" the table costs nothing at all (`:16601` is its first row
    /// that does) — but the apparent-aging threshold is a separate question and
    /// still answers for those totals.
    #[test]
    fn a_total_below_the_first_row_awards_nothing() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        for total in [-4, 0, 9] {
            let outcome = resolve_outcome(&entity, &ruleset, total).expect("aging rules");
            assert_eq!(outcome.total, total);
            assert!(outcome.awards.is_empty(), "total {total} costs nothing");
            assert!(!outcome.crisis, "total {total}");
        }

        assert!(
            !resolve_outcome(&entity, &ruleset, 0)
                .expect("aging rules")
                .apparent_age_increases
        );
        assert!(
            resolve_outcome(&entity, &ruleset, 9)
                .expect("aging rules")
                .apparent_age_increases,
            "a 9 ages the appearance without costing a point"
        );
    }

    /// A ruleset shipping no aging rules has no table to resolve against, which
    /// stands the question down rather than letting the engine invent a row.
    #[test]
    fn no_aging_rules_resolve_to_none() {
        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(without.aging().is_none());
        assert!(resolve_outcome(&living_under(&[]), &without, 22).is_none());
    }

    /// One year's submission: the die the player typed and where any open points
    /// go.
    fn request(age: u32, die: i32, distribution: &[(Characteristic, u8)]) -> AgingYearRequest {
        AgingYearRequest {
            age,
            die,
            distribution: distribution.iter().copied().collect(),
            crisis_die: None,
        }
    }

    /// A `BTreeMap` of aging points, which is how both the entity and the log
    /// entry hold them.
    fn points(entries: &[(Characteristic, u8)]) -> BTreeMap<Characteristic, u8> {
        entries.iter().copied().collect()
    }

    /// Everything the single writer writes, in one year: the row's points, the
    /// apparent age, the log entry — and nothing else.
    ///
    /// A 40-year-old working in a mine ("Work in a mine -1", `:16590`) rolls a
    /// 10: `10 + ⌈40/10⌉ - (-1) = 15`, which the table answers with "1 Aging
    /// Point in Sta" (`:16604`).
    #[test]
    fn a_resolved_year_writes_the_points_the_appearance_and_a_log_entry() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&["living_condition.work_in_a_mine"]);
        entity.birth_year = Some(1160);

        let resolved = resolve_year(&entity, &ruleset, &request(40, 10, &[]))
            .expect("an ordinary year resolves");

        assert_eq!(resolved.total.total, 15, "10 + ceil(40/10) - (-1)");
        assert_eq!(
            resolved.outcome.awards,
            vec![AgingPointAward {
                target: AgingPointTarget::Named(Characteristic::Sta),
                points: Some(1),
            }]
        );

        let applied = &resolved.entity;
        assert_eq!(applied.aging_points, points(&[(Characteristic::Sta, 1)]));
        assert_eq!(
            applied.apparent_age,
            Some(36),
            "seeded at the start age, then advanced by this year"
        );
        assert_eq!(
            applied.aging_log,
            vec![AgingLogEntry {
                year: Some(1200),
                age: Some(40),
                effect: String::new(),
                die: Some(10),
                total: Some(15),
                living_conditions: [Id::new("living_condition.work_in_a_mine")]
                    .into_iter()
                    .collect(),
                points: points(&[(Characteristic::Sta, 1)]),
                apparent_age_increased: true,
                crisis: false,
                crisis_die: None,
                crisis_total: None,
                crisis_row: None,
                crisis_severity: None,
            }],
            "the structured fields are the record; the prose is left to the player"
        );

        // Never in place: the caller's own character is exactly as it was.
        assert!(entity.aging_log.is_empty());
        assert!(entity.aging_points.is_empty());
        assert!(entity.apparent_age.is_none());
    }

    /// A year is resolved once. Applying it again would charge the character
    /// twice for a single roll, so the second attempt is refused rather than
    /// merged or silently dropped.
    #[test]
    fn the_same_year_cannot_be_resolved_twice() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        let once =
            resolve_year(&entity, &ruleset, &request(40, 10, &[])).expect("the first apply lands");
        assert_eq!(
            resolve_year(&once.entity, &ruleset, &request(40, 4, &[])).unwrap_err(),
            AgingError::YearAlreadyRecorded { age: 40 },
            "even with a different die: the year is the key"
        );

        // Its neighbours are untouched — only that one year is spoken for.
        assert!(resolve_year(&once.entity, &ruleset, &request(39, 10, &[])).is_ok());
    }

    /// "Gain sufficient Aging Points (in any Characteristics) to reach the next
    /// level in Decrepitude" (`:16602`) — the count is the table's, so a
    /// distribution that does not sum to it is refused instead of quietly
    /// awarding whatever was typed.
    #[test]
    fn a_distribution_that_misses_the_award_is_refused() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        // 9 + ceil(40/10) = 13, the first row that reaches the next Decrepitude
        // level — 5 points on the fixture's curve.
        let short = resolve_year(
            &entity,
            &ruleset,
            &request(40, 9, &[(Characteristic::Str, 2), (Characteristic::Sta, 2)]),
        );
        assert_eq!(
            short.unwrap_err(),
            AgingError::DistributionMismatch {
                owed: 5,
                distributed: 4,
            }
        );

        // An unanswered award is the same refusal, not an award of nothing.
        assert_eq!(
            resolve_year(&entity, &ruleset, &request(40, 9, &[])).unwrap_err(),
            AgingError::DistributionMismatch {
                owed: 5,
                distributed: 0,
            }
        );
    }

    /// "Gain sufficient Aging Points (**in any Characteristics**) to reach the
    /// next level in Decrepitude" (`:16602`, `:16611`) — **plural**. Reaching
    /// Decrepitude 1 costs five points, and forcing all five into one
    /// Characteristic would force Characteristic drops (`:16579`) the player may
    /// legally avoid. So the distribution is a per-Characteristic map, and this
    /// test is what stops a later refactor narrowing it to a single pick.
    #[test]
    fn a_decrepitude_award_may_be_spread_across_several_characteristics() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        let spread = resolve_year(
            &entity,
            &ruleset,
            &request(
                40,
                9,
                &[
                    (Characteristic::Str, 2),
                    (Characteristic::Sta, 2),
                    (Characteristic::Qik, 1),
                ],
            ),
        )
        .expect("five points across three Characteristics is a legal answer");

        let spread_points = points(&[
            (Characteristic::Str, 2),
            (Characteristic::Sta, 2),
            (Characteristic::Qik, 1),
        ]);
        assert_eq!(spread.entity.aging_points, spread_points);
        assert_eq!(
            spread.entity.aging_log[0].points, spread_points,
            "the log records where they went, which is what makes the year revertible"
        );
        assert!(
            spread.outcome.crisis,
            "'… and Crisis' (`:16602`) — flagged, not resolved"
        );
    }

    /// Rows 14-21 name the Characteristics themselves (`:16603-16610`), and so
    /// does every row below 10 by naming none at all. Placing points against
    /// either would invent an award the table never made.
    #[test]
    fn a_row_that_names_its_own_characteristics_takes_no_distribution() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        // 10 + ceil(40/10) = 14: "1 Aging Point in Qik" (`:16603`).
        assert_eq!(
            resolve_year(
                &entity,
                &ruleset,
                &request(40, 10, &[(Characteristic::Int, 1)])
            )
            .unwrap_err(),
            AgingError::DistributionNotOpen {
                characteristics: vec![Characteristic::Int],
            }
        );

        // And a total below the table's first row awards nothing at all.
        assert_eq!(
            resolve_year(
                &entity,
                &ruleset,
                &request(40, 1, &[(Characteristic::Int, 1)])
            )
            .unwrap_err(),
            AgingError::DistributionNotOpen {
                characteristics: vec![Characteristic::Int],
            }
        );
    }

    /// "Every Aging Point also counts as an experience point towards
    /// Decrepitude, which increases as an Ability" (`:16617`) — so the writer
    /// writes points and Decrepitude follows from them. Nothing stores the
    /// score.
    #[test]
    fn aging_points_stay_authoritative_and_decrepitude_follows_from_them() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);
        assert_eq!(decrepitude_score(&entity, &ruleset), 0);

        let resolved = resolve_year(
            &entity,
            &ruleset,
            &request(40, 9, &[(Characteristic::Str, 3), (Characteristic::Sta, 2)]),
        )
        .expect("the next Decrepitude level costs five points");

        assert_eq!(decrepitude_points_total(&resolved.entity), 5);
        assert_eq!(
            decrepitude_score(&resolved.entity, &ruleset),
            1,
            "derived from the points, with no separate write"
        );
        assert!(
            resolved.entity.decrepitude_effect.is_empty(),
            "the Decrepitude annotation is the player's prose, never the engine's"
        );
    }

    /// `apparent_age` is `None` on most characters, so the first year that needs
    /// it seeds it at [`AgingRules::start_age`] — nothing has happened before the
    /// first owed roll — and advances from there (`:16577`). Seeding once and
    /// only incrementing afterwards is what makes an out-of-order catch-up land
    /// on the same number as an in-order one.
    #[test]
    fn the_apparent_age_seeds_at_the_start_age_however_the_years_are_ordered() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        entity.age = Some(41);
        assert!(entity.apparent_age.is_none());

        let first = resolve_year(&entity, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(first.entity.apparent_age, Some(36), "35, then this year");
        let in_order =
            resolve_year(&first.entity, &ruleset, &request(41, 10, &[])).expect("a 15 at 41");
        assert_eq!(in_order.entity.apparent_age, Some(37));

        let later = resolve_year(&entity, &ruleset, &request(41, 10, &[])).expect("a 15 at 41");
        let reversed =
            resolve_year(&later.entity, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(
            reversed.entity.apparent_age, in_order.entity.apparent_age,
            "the same two years in the other order reach the same appearance"
        );

        // A hand-entered apparent age is never re-seeded, only advanced.
        let mut aged = living_under(&[]);
        aged.apparent_age = Some(50);
        assert_eq!(
            resolve_year(&aged, &ruleset, &request(40, 10, &[]))
                .expect("a 14 at 40")
                .entity
                .apparent_age,
            Some(51)
        );

        // "2 or less: No apparent aging" (`:16599`) seeds nothing at all — a
        // wealthy 40-year-old with Mild Aging rolling a 1: 1 + 4 - (2 + 1) = 2.
        let mut kept = living_under(&["living_condition.wealthy_or_healthy_location"]);
        kept.selections = vec![Selection::new(Id::new("virtue.mild_aging"))];
        let calm = resolve_year(&kept, &ruleset, &request(40, 1, &[])).expect("a 2 at 40");
        assert_eq!(calm.total.total, 2);
        assert!(
            calm.entity.apparent_age.is_none(),
            "a year that ages nothing invents no apparent age either"
        );
        assert!(!calm.entity.aging_log[0].apparent_age_increased);
    }

    /// The schedule's `recorded` flag is the log read back through
    /// [`AgingLogEntry::age`] — the key a resolved year is addressed by, since a
    /// calendar year is unavailable without a birth year.
    #[test]
    fn the_schedule_marks_a_resolved_year_as_recorded() {
        let ruleset = scheduled_ruleset();
        let mut entity = living_under(&[]);
        entity.age = Some(38);
        assert!(
            aging_schedule(&entity, &ruleset)
                .iter()
                .all(|year| !year.recorded),
            "nothing is recorded before the first apply"
        );

        let resolved = resolve_year(&entity, &ruleset, &request(37, 10, &[])).expect("a 14 at 37");
        assert_eq!(
            aging_schedule(&resolved.entity, &ruleset)
                .into_iter()
                .map(|year| (year.age, year.recorded))
                .collect::<Vec<_>>(),
            vec![(36, false), (37, true), (38, false)]
        );
    }

    /// Two refusals with nothing to award: a Decrepitude level the advancement
    /// curve cannot price (`:16617` — Decrepitude "increases as an Ability", and
    /// the table tops out), and a ruleset shipping no aging rules at all. Both
    /// are reported, never silently costed at 0.
    #[test]
    fn an_unpriceable_award_and_a_ruleset_without_aging_rules_are_refused() {
        let ruleset = scheduled_ruleset();
        let top = ruleset
            .advancement()
            .max_score()
            .expect("the fixture's curve prices something");
        let at_top = ruleset
            .advancement()
            .xp_for_score(top)
            .expect("the top score is priced");
        let mut frail = living_under(&[]);
        frail.aging_points.insert(
            Characteristic::Str,
            u8::try_from(at_top).expect("the fixture's top price fits a point count"),
        );

        assert_eq!(
            resolve_year(&frail, &ruleset, &request(40, 9, &[])).unwrap_err(),
            AgingError::AwardUnpriceable
        );

        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert_eq!(
            resolve_year(&living_under(&[]), &without, &request(40, 10, &[])).unwrap_err(),
            AgingError::NoAgingRules
        );
    }

    /// The character as a save file sees it — which is the only comparison that
    /// can witness "exact": a field-by-field check would pass over a stray key
    /// the writer left behind.
    fn saved(entity: &Entity) -> String {
        serde_json::to_string(entity).expect("a character serializes")
    }

    /// A mistyped die must be recoverable: a magus of 60 owes 25 rolls
    /// (`:2232`), and a 25-roll walk with no undo is not shippable. The widened
    /// log entry records precisely what its year did, so putting it back is
    /// exact — down to the bytes of the save.
    #[test]
    fn applying_a_year_and_reverting_it_leaves_the_character_byte_identical() {
        let ruleset = scheduled_ruleset();

        // The open-award path, on a character whose apparent age was never
        // entered: the revert has to undo the seed as well as the increment.
        let mut fresh = living_under(&["living_condition.work_in_a_mine"]);
        fresh.birth_year = Some(1160);
        let before = saved(&fresh);

        // 8 + ceil(40/10) - (-1) = 13: the next Decrepitude level, five points.
        let resolved = resolve_year(
            &fresh,
            &ruleset,
            &request(
                40,
                8,
                &[
                    (Characteristic::Str, 2),
                    (Characteristic::Sta, 2),
                    (Characteristic::Qik, 1),
                ],
            ),
        )
        .expect("a 13 at 40");
        assert_ne!(saved(&resolved.entity), before, "the year wrote something");

        let reverted =
            revert_year(&resolved.entity, &ruleset, 40).expect("the year comes back off");
        assert_eq!(saved(&reverted), before);

        // And the named-row path, on a character carrying points and an apparent
        // age of his own: those are left exactly where they were.
        let mut aged = living_under(&[]);
        aged.apparent_age = Some(44);
        aged.aging_points.insert(Characteristic::Str, 3);
        let before = saved(&aged);

        let resolved =
            resolve_year(&aged, &ruleset, &request(40, 10, &[])).expect("a 14 at 40 takes Qik");
        assert_eq!(resolved.entity.apparent_age, Some(45));
        assert_eq!(
            saved(&revert_year(&resolved.entity, &ruleset, 40).expect("reverts")),
            before
        );
    }

    /// The single writer honours `no_apparent_aging` too, which is a separate
    /// claim from [`resolve_outcome`]'s: the stored [`Entity::apparent_age`] must
    /// not move, and the year's log entry must record that it did not — otherwise
    /// [`revert_year`] would walk the appearance backwards past a year that never
    /// advanced it.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:5189, :16577.
    #[test]
    fn a_resolved_year_leaves_an_unaging_character_looking_exactly_as_he_did() {
        let ruleset = scheduled_ruleset();

        // A hand-entered apparent age stays where the player put it.
        let mut unaging = carrying("virtue.unaging");
        unaging.apparent_age = Some(30);
        let before = saved(&unaging);

        let resolved = resolve_year(&unaging, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(
            resolved.entity.apparent_age,
            Some(30),
            "'You may choose your apparent age freely' (:5189)"
        );
        assert_eq!(
            resolved.entity.aging_points,
            points(&[(Characteristic::Qik, 1)]),
            "the row's points still land — only the appearance is spared"
        );
        let entry = resolved
            .entity
            .aging_log
            .first()
            .expect("the year is logged");
        assert!(!entry.apparent_age_increased);

        // And the revert is still exact for such a character.
        assert_eq!(
            saved(&revert_year(&resolved.entity, &ruleset, 40).expect("reverts")),
            before
        );

        // With no apparent age entered, the seed never fires either.
        let fresh = carrying("virtue.unaging");
        let resolved = resolve_year(&fresh, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(resolved.entity.apparent_age, None, "nothing to seed");
    }

    /// Reverting a year no entry records is an error, not a no-op: silently
    /// doing nothing would tell the player the year had been undone.
    #[test]
    fn reverting_a_year_no_entry_records_is_refused() {
        let ruleset = scheduled_ruleset();
        let entity = living_under(&[]);

        assert_eq!(
            revert_year(&entity, &ruleset, 40).unwrap_err(),
            AgingError::YearNotRecorded { age: 40 }
        );

        let resolved = resolve_year(&entity, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(
            revert_year(&resolved.entity, &ruleset, 39).unwrap_err(),
            AgingError::YearNotRecorded { age: 39 },
            "the neighbouring year is not this one"
        );

        // And a year reverted once is no longer recorded.
        let reverted = revert_year(&resolved.entity, &ruleset, 40).expect("the first revert lands");
        assert_eq!(
            revert_year(&reverted, &ruleset, 40).unwrap_err(),
            AgingError::YearNotRecorded { age: 40 }
        );
    }

    /// A hand-written free-text entry carries no age, so it is out of the
    /// revert's reach by construction — it is removed with the ordinary log
    /// editor. Reverting a resolved year leaves it, and everything it does not
    /// own, alone.
    #[test]
    fn reverting_a_resolved_year_leaves_a_hand_written_entry_alone() {
        let ruleset = scheduled_ruleset();
        let hand_written = AgingLogEntry {
            year: Some(1180),
            effect: "A hard winter in the Rhine".to_string(),
            ..AgingLogEntry::default()
        };
        let mut entity = living_under(&[]);
        entity.aging_points.insert(Characteristic::Str, 3);
        entity.aging_log = vec![hand_written.clone()];

        let resolved = resolve_year(&entity, &ruleset, &request(40, 10, &[])).expect("a 14 at 40");
        assert_eq!(resolved.entity.aging_log.len(), 2);

        let reverted = revert_year(&resolved.entity, &ruleset, 40).expect("the resolved year");
        assert_eq!(
            reverted.aging_log,
            vec![hand_written],
            "only the entry addressed by age is removed"
        );
        assert_eq!(
            reverted.aging_points,
            points(&[(Characteristic::Str, 3)]),
            "the points the year did not award are untouched"
        );
    }

    /// "2 or less: No apparent aging" (`:16599`) — so the revert of such a year
    /// must not walk the appearance backwards. The entry's own
    /// `apparent_age_increased` is what decides, not the total.
    #[test]
    fn a_year_that_did_not_age_the_appearance_leaves_it_alone_on_revert() {
        let ruleset = scheduled_ruleset();
        // Wealthy (+2) with Mild Aging (+1) rolling a 1: 1 + 4 - 3 = 2.
        let mut kept = living_under(&["living_condition.wealthy_or_healthy_location"]);
        kept.selections = vec![Selection::new(Id::new("virtue.mild_aging"))];
        kept.apparent_age = Some(44);

        let resolved = resolve_year(&kept, &ruleset, &request(40, 1, &[])).expect("a 2 at 40");
        assert_eq!(resolved.entity.apparent_age, Some(44));

        let reverted = revert_year(&resolved.entity, &ruleset, 40).expect("reverts");
        assert_eq!(reverted.apparent_age, Some(44));
        assert!(reverted.aging_log.is_empty());
    }

    /// **CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**
    /// (`:16621`) — exactly three terms, every one of them ADDED, and every one
    /// of them reported so a sheet can show the arithmetic without re-deriving
    /// it.
    ///
    /// The age term is the aging roll's own [`AgingRules::age_modifier`], so it
    /// steps on the *first* year of a decade rather than the last and the two
    /// totals can never disagree about what "age/10 (round up)" means.
    #[test]
    fn the_crisis_total_is_a_simple_die_plus_the_age_term_plus_decrepitude() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(40), None);
        entity.aging_points.insert(Characteristic::Sta, 15);
        assert_eq!(
            decrepitude_score(&entity, &ruleset),
            2,
            "fifteen accrued Aging Points is Decrepitude 2 on the fixture's curve"
        );

        assert_eq!(
            crisis_total(&entity, &ruleset, 36, 3).expect("the fixture ships a Crisis Table"),
            CrisisTotal {
                age: 36,
                die: 3,
                age_modifier: 4,
                decrepitude_score: 2,
                total: 9,
            }
        );

        // "age/10 (round up)" steps on the first year of the decade, not the last.
        let steps: Vec<i32> = [30, 31, 39, 40, 41]
            .into_iter()
            .map(|age| {
                crisis_total(&entity, &ruleset, age, 3)
                    .expect("a Crisis Table")
                    .age_modifier
            })
            .collect();
        assert_eq!(steps, vec![3, 4, 4, 4, 5]);

        // Every term is ADDED: a bigger die and a frailer character both push the
        // total up. No term is ever subtracted — `:16621` names none.
        assert_eq!(
            crisis_total(&entity, &ruleset, 36, 10)
                .expect("a Crisis Table")
                .total,
            16
        );
        let mut frailer = entity.clone();
        frailer.aging_points.insert(Characteristic::Sta, 30);
        assert_eq!(decrepitude_score(&frailer, &ruleset), 3);
        assert_eq!(
            crisis_total(&frailer, &ruleset, 36, 3)
                .expect("a Crisis Table")
                .total,
            10
        );

        // A character who has never taken an Aging Point adds nothing for
        // Decrepitude — the term is 0, not absent.
        let unaged =
            crisis_total(&character(Some(40), None), &ruleset, 36, 3).expect("a Crisis Table");
        assert_eq!(unaged.decrepitude_score, 0);
        assert_eq!(unaged.total, 7);

        // The die is the player's and the engine does not police it: "Roll a
        // ten-sided die … a zero counts as ten" (`:474`) bounds what a UI offers,
        // not what a storyguide may hand out.
        for die in [-2, 0, 11] {
            let total = crisis_total(&entity, &ruleset, 36, die).expect("a Crisis Table");
            assert_eq!(total.die, die);
            assert_eq!(total.total, die + 4 + 2, "die {die}");
        }
    }

    /// "**Crisis:** Increase the character's Decrepitude **first**, and then roll
    /// on the Crisis Table." (`:16619`) — so the Decrepitude the total adds is the
    /// one *that year* raised, not the one the character carries today.
    ///
    /// The distinction is not academic, because this module is deliberately
    /// **order-independent**: [`resolve_year`] refuses nothing but a year already
    /// recorded, so a player may roll 36 (which flags a Crisis), carry on through
    /// 37-40, and only then resolve 36's Crisis. A live [`decrepitude_score`] read
    /// would charge that Crisis with four later years' Aging Points, which
    /// `:16619` does not license.
    #[test]
    fn the_crisis_total_reads_the_decrepitude_that_year_raised_not_todays() {
        let ruleset = crisis_ruleset();

        // Age 36: `9 + ⌈36/10⌉ = 13` reaches the next Decrepitude level and flags
        // a Crisis (`:16602`) — five points on the fixture's curve.
        let crisis_year = resolve_year(
            &character(Some(40), None),
            &ruleset,
            &request(36, 9, &[(Characteristic::Sta, 5)]),
        )
        .expect("the crisis year resolves")
        .entity;
        assert!(crisis_year.aging_log[0].crisis);
        assert_eq!(decrepitude_score(&crisis_year, &ruleset), 1);

        let when_rolled = crisis_total(&crisis_year, &ruleset, 36, 3).expect("a Crisis Table");
        assert_eq!(when_rolled.decrepitude_score, 1);
        assert_eq!(when_rolled.total, 8);

        // The player carries on before resolving the Crisis: 37 reaches the next
        // level again (ten more points), and 38-40 cost a point each.
        let mut walked = resolve_year(
            &crisis_year,
            &ruleset,
            &request(37, 9, &[(Characteristic::Sta, 10)]),
        )
        .expect("the next year resolves")
        .entity;
        for age in [38, 39, 40] {
            walked = resolve_year(
                &walked,
                &ruleset,
                &request(age, 7, &[(Characteristic::Int, 1)]),
            )
            .expect("a one-point year resolves")
            .entity;
        }
        assert_eq!(decrepitude_points_total(&walked), 18);
        assert_eq!(
            decrepitude_score(&walked, &ruleset),
            2,
            "today's Decrepitude has genuinely moved on since the crisis year"
        );

        // …and 36's Crisis is still rolled against 36's Decrepitude, to the point.
        assert_eq!(
            crisis_total(&walked, &ruleset, 36, 3).expect("a Crisis Table"),
            when_rolled
        );

        // The newest year is not special-cased the other way: a Crisis at 40 does
        // take every Aging Point the character has accrued, later years included
        // — there are none after it to leave out.
        let today = crisis_total(&walked, &ruleset, 40, 3).expect("a Crisis Table");
        assert_eq!(today.decrepitude_score, 2);
        assert_eq!(today.total, 9);
    }

    /// A legacy hand-written log entry carries neither an age nor a point
    /// distribution, so nothing about it can be dated and nothing is subtracted.
    /// A character whose whole history is hand-written therefore rolls his Crisis
    /// against every Aging Point he has accrued — and a dated year that awarded
    /// nothing subtracts nothing either.
    #[test]
    fn a_hand_written_log_never_subtracts_from_the_crisis_year() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(60), None);
        entity.aging_points.insert(Characteristic::Sta, 15);
        entity.aging_log.push(AgingLogEntry {
            effect: "Took to his bed for a month".to_string(),
            ..AgingLogEntry::default()
        });
        entity.aging_log.push(AgingLogEntry {
            age: Some(59),
            effect: "A quiet year".to_string(),
            ..AgingLogEntry::default()
        });

        let total = crisis_total(&entity, &ruleset, 36, 3).expect("a Crisis Table");
        assert_eq!(
            total.decrepitude_score, 2,
            "an undated entry dates nothing, so all fifteen points stand"
        );
        assert_eq!(total.total, 9);
    }

    /// Two rulesets with no crisis total to give: one shipping no aging rules at
    /// all, and one shipping the aging tables but no Crisis Table. Both answer
    /// `None`, exactly as [`aging_total`] does — the engine never invents a table
    /// the ruleset does not carry.
    #[test]
    fn a_ruleset_without_a_crisis_table_has_no_crisis_total() {
        let entity = character(Some(40), None);

        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(without.aging().is_none());
        assert!(crisis_total(&entity, &without, 36, 3).is_none());

        let no_crisis = scheduled_ruleset();
        assert!(no_crisis.aging().expect("aging rules").crisis.is_none());
        assert!(crisis_total(&entity, &no_crisis, 36, 3).is_none());
    }

    /// The miniature twin of `data_integrity.rs`'s shipped-catalogue lock on
    /// `:16636` — "Virtues that affect aging rolls do not affect crisis survival
    /// rolls."
    ///
    /// Both quantities the aging roll takes are witnessed moving that roll and
    /// then failing to move this one: an [`AgingEffect::AgingRoll`] modifier and
    /// an [`AgingEffect::LivingConditions`] one. Mild Aging is the proof case,
    /// because `:4530` grants a Living Conditions +1 and a crisis-survival +3 in
    /// one sentence and exactly one of them belongs here.
    #[test]
    fn an_aging_roll_modifier_never_reaches_the_crisis_survival_total() {
        let ruleset = crisis_ruleset();
        let illness = crisis_outcome(&ruleset, "crisis.minor_illness");

        let mut walled = character(Some(40), None);
        walled.selections = vec![
            Selection::new(Id::new("virtue.faerie_blood")),
            Selection::new(Id::new("flaw.poor_living_conditions")),
        ];

        // Both modifiers demonstrably move the AGING TOTAL …
        let aging = aging_total(&walled, &ruleset, 40, 6).expect("the fixture ships aging rules");
        assert_eq!(aging.trait_modifier, -1, "Faerie Blood's aging-roll -1");
        assert_eq!(
            aging.living_conditions.total, -1,
            "Poor Living Conditions' -1"
        );

        // … and neither one reaches the survival roll.
        let survival =
            crisis_survival(&walled, &ruleset, &illness).expect("an illness is survivable");
        assert_eq!(survival.modifiers, vec![]);
        assert_eq!(survival.modifier_total, 0);
        // The roll itself is the row's, reported and not resolved.
        assert_eq!(survival.ease_factor, Some(3));
        assert_eq!(survival.ritual_level, 20);

        // Mild Aging's two halves part company here: the +3 arrives, the +1 does
        // not, and the +1 is still doing its work on the other roll.
        let mut mild = walled.clone();
        mild.selections
            .push(Selection::new(Id::new("virtue.mild_aging")));
        let survival =
            crisis_survival(&mild, &ruleset, &illness).expect("an illness is survivable");
        assert_eq!(
            survival.modifiers,
            vec![CrisisModifier {
                source: CrisisModifierSource::Trait {
                    item: Id::new("virtue.mild_aging"),
                },
                amount: 3,
            }],
            "only the crisis-survival grant of :4530 is a survival modifier"
        );
        assert_eq!(survival.modifier_total, 3);
        assert_eq!(
            living_conditions_modifier(&mild, &ruleset).total,
            0,
            "the Flaw's -1 and Mild Aging's +1 cancel, on the aging roll where they belong"
        );
    }

    /// The Bronze cord applies "to rolls to resist aging" (`:10844`), and the
    /// roll that names is the crisis *survival* roll — an aging roll is not a
    /// roll one passes or fails. So the cord goes in here and stays out of the
    /// AGING TOTAL, and this test watches both directions at once.
    #[test]
    fn the_bronze_cord_reaches_crisis_survival_and_never_the_aging_total() {
        let ruleset = crisis_ruleset();
        let illness = crisis_outcome(&ruleset, "crisis.minor_illness");
        let mut magus = character(Some(40), None);

        let before = aging_total(&magus, &ruleset, 40, 6).expect("aging rules");
        assert!(
            crisis_survival(&magus, &ruleset, &illness)
                .expect("survivable")
                .modifiers
                .is_empty(),
            "a magus with no familiar carries no cord to name"
        );

        magus.familiar = Some(Familiar {
            name: "Corax".to_string(),
            cord_bronze: 2,
            ..Default::default()
        });

        let survival =
            crisis_survival(&magus, &ruleset, &illness).expect("an illness is survivable");
        assert_eq!(
            survival.modifiers,
            vec![CrisisModifier {
                source: CrisisModifierSource::BronzeCord,
                amount: 2,
            }]
        );
        assert_eq!(survival.modifier_total, 2);

        let after = aging_total(&magus, &ruleset, 40, 6).expect("aging rules");
        assert_eq!(
            after, before,
            "the cord moves no term of the AGING TOTAL (:16636)"
        );
    }

    /// "8 or less — Bedridden for a week" and "9-14 — Bedridden for a month."
    /// (`:16626`, `:16627`) cost nothing but time: no Stamina roll, no Ritual,
    /// nothing for a modifier to modify. So there is no survival read-out to
    /// give, and a character loaded with every modifier in the fixture still
    /// gets none.
    #[test]
    fn a_bedridden_outcome_has_no_survival_roll() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(40), None);
        entity.selections = vec![Selection::new(Id::new("virtue.mild_aging"))];
        entity.familiar = Some(Familiar {
            name: "Corax".to_string(),
            cord_bronze: 3,
            ..Default::default()
        });

        for id in ["crisis.bedridden_week", "crisis.bedridden_month"] {
            assert!(
                crisis_survival(&entity, &ruleset, &crisis_outcome(&ruleset, id)).is_none(),
                "'{id}' is time, not a roll"
            );
        }
    }

    /// The doctor of `:16634` is *what the rules permit*, not a number the engine
    /// adds — and a ruleset that ships no attendant permits none. The fixture is
    /// deliberately one such ruleset; the shipped table's attendant is asserted
    /// against its own values in `data_integrity.rs`.
    #[test]
    fn a_ruleset_that_ships_no_attendant_allows_no_doctor() {
        let ruleset = crisis_ruleset();
        assert!(
            ruleset
                .aging()
                .expect("aging rules")
                .crisis
                .as_ref()
                .expect("a Crisis Table")
                .attendant
                .is_none(),
            "the fixture is the no-attendant case"
        );

        let survival = crisis_survival(
            &character(Some(40), None),
            &ruleset,
            &crisis_outcome(&ruleset, "crisis.minor_illness"),
        )
        .expect("an illness is survivable");
        assert_eq!(survival.allowances, vec![]);
    }

    /// The Crisis Table is indexed by the CRISIS TOTAL (`:16621`) and answers for
    /// **every** integer: its first row is open below ("8 or less", `:16626`) and
    /// its last open above ("19+", `:16632`), so a total no die could reach lands
    /// on a row exactly as a middling one does.
    ///
    /// The row comes back whole rather than as a copied outcome, because the id is
    /// what the UI keys the row's display text off — the text lives in
    /// `rules/i18n/<lang>/aging.json` and never in the engine.
    #[test]
    fn a_crisis_total_lands_on_the_row_whose_band_covers_it() {
        let ruleset = crisis_ruleset();

        let landings: Vec<(i32, &str)> = [-40, 0, 8, 9, 14, 15, 16, 400]
            .into_iter()
            .map(|total| {
                let row = resolve_crisis_row(&ruleset, total)
                    .unwrap_or_else(|| panic!("the fixture's table covers {total}"));
                (total, row.id.as_str())
            })
            .collect();
        assert_eq!(
            landings,
            vec![
                (-40, "crisis.bedridden_week"),
                (0, "crisis.bedridden_week"),
                (8, "crisis.bedridden_week"),
                (9, "crisis.bedridden_month"),
                (14, "crisis.bedridden_month"),
                (15, "crisis.minor_illness"),
                (16, "crisis.terminal_illness"),
                (400, "crisis.terminal_illness"),
            ]
        );

        // The row carries its outcome, so the look-up is all a caller needs to
        // reach the survival read-out.
        assert_eq!(
            resolve_crisis_row(&ruleset, 15)
                .expect("15 lands on the minor illness")
                .outcome,
            CrisisOutcome::Illness {
                severity: CrisisSeverity::Minor,
                ease_factor: Some(3),
                ritual_level: 20,
            }
        );
    }

    /// A ruleset with no aging rules, and one whose aging block ships no Crisis
    /// Table, both resolve no row at all — the engine never invents a table the
    /// ruleset does not carry, exactly as [`crisis_total`] never invents a total.
    #[test]
    fn a_ruleset_without_a_crisis_table_resolves_no_crisis_row() {
        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(without.aging().is_none());
        assert!(resolve_crisis_row(&without, 15).is_none());

        let no_crisis = scheduled_ruleset();
        assert!(no_crisis.aging().expect("aging rules").crisis.is_none());
        assert!(resolve_crisis_row(&no_crisis, 15).is_none());
    }

    /// One Crisis end to end, as a caller receives it: the CRISIS TOTAL broken
    /// into its three terms (`:16621`), the row that total lands on
    /// (`:16624-16632`), what the row costs, and what surviving it would take
    /// (`:16628-16638`).
    ///
    /// And **nothing is written**. The Crisis is a reading, not a resolution:
    /// `aging_points` stays where `resolve_year` left it, no log entry appears,
    /// and the character comes out of the preview byte-identical to the one that
    /// went in.
    #[test]
    fn the_crisis_preview_composes_the_total_the_row_and_the_survival_roll() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(40), None);
        entity.aging_points.insert(Characteristic::Sta, 15);
        assert_eq!(decrepitude_score(&entity, &ruleset), 2);
        let before = saved(&entity);

        // `9 + ⌈36/10⌉ + 2 = 15` — the minor illness of `:16628`.
        let preview = crisis_preview(&entity, &ruleset, 36, 9).expect("the fixture ships a table");
        assert_eq!(
            preview.total,
            CrisisTotal {
                age: 36,
                die: 9,
                age_modifier: 4,
                decrepitude_score: 2,
                total: 15,
            }
        );
        assert_eq!(preview.row, Id::new("crisis.minor_illness"));
        assert_eq!(
            preview.outcome,
            CrisisOutcome::Illness {
                severity: CrisisSeverity::Minor,
                ease_factor: Some(3),
                ritual_level: 20,
            }
        );
        let survival = preview.survival.expect("an illness is survivable");
        assert_eq!(survival.ease_factor, Some(3));
        assert_eq!(survival.ritual_level, 20);
        assert_eq!(survival.modifier_total, 0);

        // The terminal row offers no Stamina roll at all (`:16632`) — an absent
        // Ease Factor, and only the Ritual level can answer.
        let terminal = crisis_preview(&entity, &ruleset, 36, 10).expect("a table");
        assert_eq!(terminal.total.total, 16);
        assert_eq!(terminal.row, Id::new("crisis.terminal_illness"));
        let survival = terminal.survival.expect("an illness is survivable");
        assert_eq!(survival.ease_factor, None);
        assert_eq!(survival.ritual_level, 40);

        assert_eq!(saved(&entity), before, "a preview writes nothing");
    }

    /// "8 or less — Bedridden for a week" and "9-14 — Bedridden for a month."
    /// (`:16626`, `:16627`) are time, not a roll. So the composed read-out names
    /// the row and carries **no** survival read-out — an empty one would read as
    /// "survivable on a 0".
    #[test]
    fn a_bedridden_crisis_preview_carries_no_survival_read_out() {
        let ruleset = crisis_ruleset();
        let entity = character(Some(40), None);

        // No Decrepitude yet, so `die + ⌈36/10⌉` is the whole total.
        let week = crisis_preview(&entity, &ruleset, 36, 4).expect("a table");
        assert_eq!(week.total.total, 8);
        assert_eq!(week.row, Id::new("crisis.bedridden_week"));
        assert_eq!(week.outcome, CrisisOutcome::Bedridden);
        assert!(week.survival.is_none(), "a week in bed is not a roll");

        let month = crisis_preview(&entity, &ruleset, 36, 5).expect("a table");
        assert_eq!(month.total.total, 9);
        assert_eq!(month.row, Id::new("crisis.bedridden_month"));
        assert!(month.survival.is_none(), "a month in bed is not a roll");
    }

    /// The composed read-out invents no table either: with no aging rules, or
    /// aging rules carrying no Crisis Table, there is no Crisis to read.
    #[test]
    fn a_ruleset_without_a_crisis_table_has_no_crisis_preview() {
        let entity = character(Some(40), None);

        let without = Ruleset::from_json("test", "1", "[]", "[]").expect("an empty ruleset loads");
        assert!(crisis_preview(&entity, &without, 36, 9).is_none());

        let no_crisis = scheduled_ruleset();
        assert!(crisis_preview(&entity, &no_crisis, 36, 9).is_none());
    }

    /// `:16636` — "Virtues that affect aging rolls do not affect crisis survival
    /// rolls" — held through the **composed** path, which is a second way for an
    /// aging-roll modifier to leak: the read-out that carries the total and the
    /// survival roll in one value could sum them into either.
    ///
    /// Neither half moves. The three terms of `:16621` are all the total has, and
    /// only the survival grant `:4530` makes by name reaches the survival roll.
    #[test]
    fn a_crisis_preview_leaks_no_aging_roll_modifier_into_either_half() {
        let ruleset = crisis_ruleset();
        let mut loaded = character(Some(40), None);
        loaded.selections = vec![
            Selection::new(Id::new("virtue.faerie_blood")),
            Selection::new(Id::new("flaw.poor_living_conditions")),
            Selection::new(Id::new("virtue.mild_aging")),
        ];

        // Every one of those modifiers demonstrably moves the AGING TOTAL.
        let aging = aging_total(&loaded, &ruleset, 36, 9).expect("aging rules");
        assert_eq!(aging.trait_modifier, -1, "Faerie Blood's aging-roll -1");
        assert_eq!(aging.living_conditions.total, 0, "the -1 and the +1 cancel");

        let preview = crisis_preview(&loaded, &ruleset, 36, 9).expect("a table");
        assert_eq!(
            preview.total,
            CrisisTotal {
                age: 36,
                die: 9,
                age_modifier: 4,
                decrepitude_score: 0,
                total: 13,
            },
            "the CRISIS TOTAL is the three terms of :16621 and no fourth"
        );
        assert_eq!(preview.row, Id::new("crisis.bedridden_month"));

        // On a row that does call for a survival roll, only the +3 of :4530 that
        // names this roll arrives.
        let illness = crisis_preview(&loaded, &ruleset, 36, 11).expect("a table");
        assert_eq!(illness.row, Id::new("crisis.minor_illness"));
        let survival = illness.survival.expect("an illness is survivable");
        assert_eq!(
            survival.modifiers,
            vec![CrisisModifier {
                source: CrisisModifierSource::Trait {
                    item: Id::new("virtue.mild_aging"),
                },
                amount: 3,
            }]
        );
        assert_eq!(survival.modifier_total, 3);
    }

    /// The same submission, plus the Simple Die the player threw at the Crisis
    /// Table.
    fn crisis_request(
        age: u32,
        die: i32,
        distribution: &[(Characteristic, u8)],
        crisis_die: i32,
    ) -> AgingYearRequest {
        AgingYearRequest {
            crisis_die: Some(crisis_die),
            ..request(age, die, distribution)
        }
    }

    /// `:16619`'s ordering, written into the writer: "**Crisis:** Increase the
    /// character's Decrepitude first, and then roll on the Crisis Table."
    ///
    /// A 40-year-old with nothing accrued rolls a 9: `9 + ⌈40/10⌉ = 13`, the row
    /// that reaches the next Decrepitude level **and** sends him to the Crisis
    /// Table. Five points take him to Decrepitude 1 on the fixture's curve, and
    /// the CRISIS TOTAL is then `7 + 4 + 1 = 12` — the **1** is the whole point.
    /// Read before the award it would have been 0, and the Crisis would have been
    /// rolled against a score the year had already left behind.
    #[test]
    fn a_resolved_crisis_year_records_the_roll_the_row_and_its_severity() {
        let ruleset = crisis_ruleset();
        let entity = character(Some(40), None);

        let resolved = resolve_year(
            &entity,
            &ruleset,
            &crisis_request(40, 9, &[(Characteristic::Sta, 5)], 7),
        )
        .expect("a crisis year resolves");

        assert_eq!(resolved.total.total, 13);
        assert!(resolved.outcome.crisis, "the row calls for one");

        let crisis = resolved.crisis.as_ref().expect("and the player rolled it");
        assert_eq!(
            crisis.total,
            CrisisTotal {
                age: 40,
                die: 7,
                age_modifier: 4,
                decrepitude_score: 1,
                total: 12,
            },
            "the Decrepitude this very year raised, not the one it started with"
        );
        assert_eq!(crisis.row, Id::new("crisis.bedridden_month"));
        assert_eq!(crisis.outcome, CrisisOutcome::Bedridden);
        assert!(crisis.survival.is_none(), "a month in bed is not a roll");

        // And the year records it, so the save carries what happened.
        let applied = &resolved.entity;
        assert_eq!(
            applied.aging_log,
            vec![AgingLogEntry {
                year: None,
                age: Some(40),
                effect: String::new(),
                die: Some(9),
                total: Some(13),
                living_conditions: std::collections::BTreeSet::new(),
                points: points(&[(Characteristic::Sta, 5)]),
                apparent_age_increased: true,
                crisis: true,
                crisis_die: Some(7),
                crisis_total: Some(12),
                crisis_row: Some(Id::new("crisis.bedridden_month")),
                crisis_severity: None,
            }],
            "a bedridden row has no severity: it is time, not an illness"
        );

        // The points stay authoritative and the score still follows from them.
        assert_eq!(applied.aging_points, points(&[(Characteristic::Sta, 5)]));
        assert_eq!(decrepitude_score(applied, &ruleset), 1);

        // Never in place.
        assert!(entity.aging_log.is_empty());
        assert!(entity.aging_points.is_empty());
    }

    /// The severity of an illness row is recorded beside its id (`:16628-16632`),
    /// and the heaviest row the table has still hands back a **living** character.
    ///
    /// The engine reports what surviving would take — the Creo Corpus level of
    /// `:16638`, and the Ease Factor where the row offers a roll at all — and then
    /// stops. It throws no Stamina die and kills nobody; that is the table's to
    /// decide and the player's to record.
    #[test]
    fn a_terminal_crisis_is_recorded_and_kills_nobody() {
        let ruleset = crisis_ruleset();
        let entity = character(Some(40), None);

        // `12 + ⌈40/10⌉ + 1 = 17`, above the fixture's open-ended terminal row.
        let resolved = resolve_year(
            &entity,
            &ruleset,
            &crisis_request(40, 9, &[(Characteristic::Sta, 5)], 12),
        )
        .expect("a crisis year resolves");

        let crisis = resolved.crisis.as_ref().expect("the player rolled it");
        assert_eq!(crisis.total.total, 17);
        assert_eq!(crisis.row, Id::new("crisis.terminal_illness"));
        assert_eq!(
            crisis.outcome,
            CrisisOutcome::Illness {
                severity: CrisisSeverity::Terminal,
                ease_factor: None,
                ritual_level: 40,
            }
        );
        let survival = crisis.survival.as_ref().expect("an illness is survivable");
        assert_eq!(survival.ease_factor, None, "`:16632` offers no roll");
        assert_eq!(survival.ritual_level, 40);

        let entry = &resolved.entity.aging_log[0];
        assert_eq!(entry.crisis_row, Some(Id::new("crisis.terminal_illness")));
        assert_eq!(entry.crisis_severity, Some(CrisisSeverity::Terminal));

        // Alive, and holding exactly the points the aging row awarded.
        assert_eq!(
            resolved.entity.aging_points,
            points(&[(Characteristic::Sta, 5)])
        );
    }

    /// A Crisis the table demanded and nobody has rolled yet is a state of its
    /// own, and the year is still recorded. The player may not have the Simple Die
    /// to hand, and refusing to write down the aging roll he *did* make would lose
    /// the one thing that actually happened.
    ///
    /// So `crisis: true` with no `crisis_row` means owed-and-unrolled, and a
    /// ruleset shipping no Crisis Table leaves every crisis year in exactly that
    /// state however many dice are typed.
    #[test]
    fn a_crisis_nobody_has_rolled_stays_owed_on_the_year() {
        let ruleset = crisis_ruleset();
        let entity = character(Some(40), None);

        let unrolled = resolve_year(
            &entity,
            &ruleset,
            &request(40, 9, &[(Characteristic::Sta, 5)]),
        )
        .expect("the aging year resolves on its own");
        assert!(unrolled.crisis.is_none(), "no die, no Crisis Table read");
        let entry = &unrolled.entity.aging_log[0];
        assert!(entry.crisis, "but the year still demanded one");
        assert_eq!(entry.crisis_die, None);
        assert_eq!(entry.crisis_total, None);
        assert_eq!(entry.crisis_row, None);
        assert_eq!(entry.crisis_severity, None);

        // A ruleset with no Crisis Table has no Crisis to resolve, die or no die.
        let no_table = scheduled_ruleset();
        let owed = resolve_year(
            &entity,
            &no_table,
            &crisis_request(40, 9, &[(Characteristic::Sta, 5)], 7),
        )
        .expect("the aging year resolves");
        assert!(owed.crisis.is_none());
        assert!(owed.entity.aging_log[0].crisis);
        assert_eq!(owed.entity.aging_log[0].crisis_row, None);
    }

    /// `:16636` — "Virtues that affect aging rolls do not affect crisis survival
    /// rolls" — held through the **writer**, which is the third and worst place it
    /// could leak: one call now computes the AGING TOTAL, which *does* take the
    /// trait modifiers, and the CRISIS TOTAL, which takes none, off one character.
    /// A single shared `entity` between the two halves is exactly the shape that
    /// invites someone to "reuse" the modifier.
    ///
    /// The character wears all three kinds at once. Faerie Blood's aging-roll `-1`
    /// and the two Living Conditions modifiers demonstrably move the aging half;
    /// the CRISIS TOTAL is still the three terms of `:16621`; and only Mild
    /// Aging's `+3`, which names the survival roll (`:4530`), reaches the survival
    /// read-out the year hands back.
    #[test]
    fn a_resolved_crisis_year_leaks_no_aging_roll_modifier_into_the_crisis() {
        let ruleset = crisis_ruleset();
        let mut loaded = character(Some(40), None);
        loaded.selections = vec![
            Selection::new(Id::new("virtue.faerie_blood")),
            Selection::new(Id::new("flaw.poor_living_conditions")),
            Selection::new(Id::new("virtue.mild_aging")),
        ];

        // `10 + ⌈40/10⌉ - 0 + (-1) = 13`, the Crisis row.
        let resolved = resolve_year(
            &loaded,
            &ruleset,
            &crisis_request(40, 10, &[(Characteristic::Sta, 5)], 10),
        )
        .expect("a crisis year resolves");
        assert_eq!(resolved.total.total, 13);
        assert_eq!(
            resolved.total.trait_modifier, -1,
            "the aging half does take Faerie Blood's -1"
        );
        assert_eq!(
            resolved.total.living_conditions.total, 0,
            "-1 and +1 cancel"
        );

        let crisis = resolved.crisis.as_ref().expect("the player rolled it");
        assert_eq!(
            crisis.total,
            CrisisTotal {
                age: 40,
                die: 10,
                age_modifier: 4,
                decrepitude_score: 1,
                total: 15,
            },
            "the CRISIS TOTAL is the three terms of :16621 and no fourth"
        );
        assert_eq!(crisis.row, Id::new("crisis.minor_illness"));

        let survival = crisis.survival.as_ref().expect("an illness is survivable");
        assert_eq!(
            survival.modifiers,
            vec![CrisisModifier {
                source: CrisisModifierSource::Trait {
                    item: Id::new("virtue.mild_aging"),
                },
                amount: 3,
            }],
            "only the grant :4530 makes to THIS roll by name"
        );
        assert_eq!(survival.modifier_total, 3);

        // And the year records the total it actually rolled against.
        assert_eq!(resolved.entity.aging_log[0].crisis_total, Some(15));
    }

    /// `revert_year` stays **exact** across the Crisis leg, which is the property
    /// a pre-play catch-up of 25 rolls (`:2232`) depends on and the one a new leg
    /// is most likely to break.
    ///
    /// The year under test writes everything the leg can write: the row's Aging
    /// Points (and so the Decrepitude the Crisis was rolled against), an advanced
    /// apparent age, and a log entry carrying the Simple Die, the CRISIS TOTAL, the
    /// row and its severity. Taking it back off restores the save **byte for
    /// byte** — the Longevity Ritual included, which costs nothing precisely
    /// because the leg reports it spent rather than deleting it.
    ///
    /// Nothing new was needed in [`revert_year`] to make this true, and that is
    /// the claim: the Crisis is recorded *inside* the year's own entry, and the
    /// entry is what the revert removes.
    #[test]
    fn reverting_a_resolved_crisis_year_leaves_the_character_byte_identical() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(40), Some(1160));
        with_ritual(&mut entity, Some(4));
        entity.aging_points.insert(Characteristic::Str, 3);
        entity.apparent_age = Some(38);
        let before = saved(&entity);

        // `13 + ⌈40/10⌉ - 4 = 13`: the Crisis row. Three points are already
        // accrued, so two more reach Decrepitude 1 — and `12 + 4 + 1 = 17` is the
        // terminal row.
        let resolved = resolve_year(
            &entity,
            &ruleset,
            &crisis_request(40, 13, &[(Characteristic::Sta, 2)], 12),
        )
        .expect("a crisis year resolves");

        let entry = &resolved.entity.aging_log[0];
        assert_eq!(entry.crisis_die, Some(12));
        assert_eq!(entry.crisis_total, Some(17));
        assert_eq!(entry.crisis_row, Some(Id::new("crisis.terminal_illness")));
        assert_eq!(entry.crisis_severity, Some(CrisisSeverity::Terminal));
        assert_eq!(resolved.entity.apparent_age, Some(39));
        assert_eq!(resolved.notes, vec![AgingNote::LongevityRitualSpent]);
        assert_ne!(saved(&resolved.entity), before, "the year wrote something");

        let reverted =
            revert_year(&resolved.entity, &ruleset, 40).expect("the year comes back off");
        assert_eq!(saved(&reverted), before);
        assert!(
            reverted.longevity_ritual.is_some(),
            "a ritual reported spent is a ritual still there to come back"
        );
    }

    /// `:16573`, which is a rule about a **stored choice** and therefore a rule
    /// about what the engine must not quietly do to one.
    ///
    /// > A Longevity Ritual is effective until the character suffers a crisis.
    /// > When the crisis occurs, the ritual assures that the character survives,
    /// > but its power is spent, and the focal ritual must be performed again.
    ///
    /// So the year reports that the ritual is spent, and the ritual stays exactly
    /// where the player put it. Clearing [`Entity::longevity_ritual`] would delete
    /// a recorded bonus and a hand-typed focus that "must be repeated" (`:10668`)
    /// when the ritual is performed again — and would make the year unrevertible
    /// into the bargain.
    ///
    /// The note follows the **Crisis**, not the Crisis *roll*: "when the crisis
    /// occurs" is the row's doing (`:16602`, `:16611`), and the Simple Die only
    /// decides how bad it was. So a Crisis owed and unrolled spends the ritual too.
    #[test]
    fn a_crisis_spends_the_longevity_ritual_and_never_deletes_it() {
        let ruleset = crisis_ruleset();
        let mut entity = character(Some(40), None);
        with_ritual(&mut entity, Some(4));
        let before = saved(&entity);

        // `9 + ⌈40/10⌉ - 4 = 9` would not reach the crisis row, so the ritual is
        // given no bonus to subtract here and 13 stands.
        let mut holder = entity.clone();
        with_ritual(&mut holder, None);
        let resolved = resolve_year(
            &holder,
            &ruleset,
            &crisis_request(40, 9, &[(Characteristic::Sta, 5)], 7),
        )
        .expect("a crisis year resolves");

        assert!(resolved.crisis.is_some());
        assert_eq!(resolved.notes, vec![AgingNote::LongevityRitualSpent]);
        assert_eq!(
            resolved.entity.longevity_ritual, holder.longevity_ritual,
            "the ritual is reported spent, never deleted"
        );

        // Unrolled, the Crisis has still occurred, so the ritual is still spent.
        let unrolled = resolve_year(
            &holder,
            &ruleset,
            &request(40, 9, &[(Characteristic::Sta, 5)]),
        )
        .expect("the aging year resolves on its own");
        assert!(unrolled.crisis.is_none());
        assert_eq!(unrolled.notes, vec![AgingNote::LongevityRitualSpent]);

        // A year with no Crisis spends nothing, and neither does a Crisis suffered
        // by a character who holds no ritual.
        let quiet = resolve_year(
            &holder,
            &ruleset,
            &crisis_request(40, 6, &[(Characteristic::Str, 1)], 7),
        )
        .expect("an ordinary year resolves");
        assert!(quiet.notes.is_empty(), "no Crisis, nothing spent");

        let ritualless = resolve_year(
            &character(Some(40), None),
            &ruleset,
            &crisis_request(40, 9, &[(Characteristic::Sta, 5)], 7),
        )
        .expect("a crisis year resolves");
        assert!(ritualless.notes.is_empty(), "nothing to spend");

        assert_eq!(
            saved(&entity),
            before,
            "the caller's character is untouched"
        );
    }

    /// Whether a Crisis happened is the **table's** call (`:16602`, `:16611`), not
    /// the player's. A Simple Die typed against a year the aging table never sent
    /// to the Crisis Table therefore resolves nothing — it is not an error,
    /// because nothing is written from it and no roll disagrees with the sheet.
    #[test]
    fn a_crisis_die_never_invents_a_crisis_the_table_did_not_call_for() {
        let ruleset = crisis_ruleset();
        let entity = character(Some(40), None);

        // `6 + ⌈40/10⌉ = 10` — one Aging Point, and no Crisis.
        let resolved = resolve_year(
            &entity,
            &ruleset,
            &crisis_request(40, 6, &[(Characteristic::Str, 1)], 7),
        )
        .expect("an ordinary year resolves");

        assert!(!resolved.outcome.crisis);
        assert!(resolved.crisis.is_none());
        let entry = &resolved.entity.aging_log[0];
        assert!(!entry.crisis);
        assert_eq!(entry.crisis_die, None, "an unused die is not recorded");
        assert_eq!(entry.crisis_total, None);
        assert_eq!(entry.crisis_row, None);
    }
}
