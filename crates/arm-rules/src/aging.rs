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
use crate::ruleset::Ruleset;
use crate::types::{Entity, Id, SourceRef, is_false};

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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16565, :16575, :2232.
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
    use crate::types::{Entity, EntityKind, RulesetRef};
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
    fn scheduled_ruleset() -> Ruleset {
        let aging = r#"{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          "longevity_clamp": { "max_total": 9, "until_age": 35 },
          "living_conditions": [
            { "id": "living_condition.average_peasant", "modifier": 0 }
          ],
          "outcomes": [
            { "min": 10, "max": 21, "effect": { "type": "any_characteristic", "points": 1 } },
            { "min": 22, "effect": { "type": "next_decrepitude_level_and_crisis" } }
          ]
        }"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
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
}
