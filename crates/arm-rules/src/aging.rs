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
//! Crisis resolution — the Crisis Table and its survival rolls
//! (`:16619-16632`) — is deliberately absent. An aging row can *send* a
//! character to a crisis
//! ([`AgingRowEffect::NextDecrepitudeLevelAndCrisis`]), but resolving one is its
//! own slice.
//!
//! See `RULES.md` for the provenance of every value the shipped
//! `rules/core/aging.json` carries.

use serde::{Deserialize, Serialize};

use crate::characteristics::Characteristic;
use crate::effective::selections_for_effects;
use crate::ruleset::Ruleset;
use crate::types::{AgingEffect, Effect, Entity, Id, SourceRef, is_false};

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
/// **No `recorded: bool` here, deliberately.** Deciding whether a year is already
/// logged needs the widened `AgingLogEntry` a later step introduces; today the log
/// carries only `{ year, effect }`, so an undated character's entries could not be
/// matched to a schedule row at all. A flag added now would be a half-answer that
/// step would have to redefine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingYear {
    /// The age the character reaches in this year of the schedule.
    pub age: u32,
    /// The calendar year that age falls in — `birth_year + age` — or `None` when
    /// the character has no birth year recorded.
    pub year: Option<i32>,
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
            year: entity
                .birth_year
                .zip(i32::try_from(age).ok())
                .map(|(birth, elapsed)| birth.saturating_add(elapsed)),
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
                // Neither is a term of the total: `no_aging` decides whether
                // Aging Points reach the Characteristics at all, and
                // `decrepitude` modifies the accrued score, not the roll.
                AgingEffect::NoAging | AgingEffect::Decrepitude => {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::types::{
        Entity, EntityKind, LongevityRitual, LongevitySource, RulesetRef, Selection,
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
            { "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
            { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }
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
            "effects": [{ "type": "aging_mod", "kind": "no_aging", "amount": 0 }] },
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
            aging: Some(aging),
            ..RulesetSources::default()
        })
        .expect("the aging fixture loads")
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
                    year: Some(1196)
                },
                AgingYear {
                    age: 37,
                    year: Some(1197)
                },
                AgingYear {
                    age: 38,
                    year: Some(1198)
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
}
