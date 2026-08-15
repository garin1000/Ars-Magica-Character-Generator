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
use crate::types::{Id, SourceRef, is_false};

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
}
