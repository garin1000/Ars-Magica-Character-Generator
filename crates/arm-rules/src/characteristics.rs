//! Characteristics: the eight inborn attributes every character has, and the
//! point-buy cost table that governs them at character creation.
//!
//! The eight Characteristics are a fixed enum (the rules name exactly these
//! eight). The point-buy *cost table* is data, loaded into the [`Ruleset`] from
//! `rules/core/characteristics.json` — it is never hardcoded here, so the rule
//! numbers live with the other mechanics.
//!
//! [`Ruleset`]: crate::ruleset::Ruleset

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// The eight inborn Characteristics.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:1023-1025 ("There are
/// eight Characteristics in Ars Magica").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Characteristic {
    /// Intelligence.
    Int,
    /// Perception.
    Per,
    /// Strength.
    Str,
    /// Stamina.
    Sta,
    /// Presence.
    Pre,
    /// Communication.
    Com,
    /// Dexterity.
    Dex,
    /// Quickness.
    Qik,
}

impl Characteristic {
    /// All eight Characteristics, in canonical order.
    pub const ALL: [Characteristic; 8] = [
        Characteristic::Int,
        Characteristic::Per,
        Characteristic::Str,
        Characteristic::Sta,
        Characteristic::Pre,
        Characteristic::Com,
        Characteristic::Dex,
        Characteristic::Qik,
    ];
}

impl fmt::Display for Characteristic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Characteristic::Int => "int",
            Characteristic::Per => "per",
            Characteristic::Str => "str",
            Characteristic::Sta => "sta",
            Characteristic::Pre => "pre",
            Characteristic::Com => "com",
            Characteristic::Dex => "dex",
            Characteristic::Qik => "qik",
        })
    }
}

/// One row of the Characteristic point-buy table: the point `cost` to set a
/// Characteristic to `score`. A positive cost spends points; a negative cost is
/// the rulebook's "Gain N" rows (e.g. score −1 has cost −1, i.e. gains 1 point).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacteristicCost {
    /// The Characteristic score this row prices.
    pub score: i8,
    /// Points spent (positive) or gained (negative) to reach `score`.
    pub cost: i8,
}

/// The Characteristic point-buy rules: a starting point pool and the cost table.
///
/// Loaded as data from `rules/core/characteristics.json`. The legal score range
/// is derived from the table rows, not hardcoded.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2340-2354.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacteristicRules {
    /// Points available to spend at creation (the rulebook's "seven points").
    pub start_points: u8,
    /// The cost table, one row per legal score.
    pub costs: Vec<CharacteristicCost>,
}

impl CharacteristicRules {
    /// The point cost to set a Characteristic to `score`, or `None` if `score`
    /// is outside the table (an illegal score).
    pub fn cost_for(&self, score: i8) -> Option<i8> {
        self.costs
            .iter()
            .find(|row| row.score == score)
            .map(|row| row.cost)
    }

    /// The total point cost of a set of Characteristic scores. Scores outside the
    /// table contribute 0 (they are reported separately as out-of-range), so this
    /// reflects only the legal spend.
    pub fn total_cost(&self, scores: &BTreeMap<Characteristic, i8>) -> i32 {
        scores
            .values()
            .map(|&score| self.cost_for(score).unwrap_or(0) as i32)
            .sum()
    }

    /// The lowest legal score in the table, if any.
    pub fn min_score(&self) -> Option<i8> {
        self.costs.iter().map(|row| row.score).min()
    }

    /// The highest legal score in the table, if any.
    pub fn max_score(&self) -> Option<i8> {
        self.costs.iter().map(|row| row.score).max()
    }

    /// `true` if `score` has a row in the cost table.
    pub fn is_legal_score(&self, score: i8) -> bool {
        self.cost_for(score).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// The canonical ArM5 cost table (Core Rules 2346-2354), used across tests.
    fn rules() -> CharacteristicRules {
        serde_json::from_str(
            r#"{
              "start_points": 7,
              "costs": [
                { "score": 3, "cost": 6 },
                { "score": 2, "cost": 3 },
                { "score": 1, "cost": 1 },
                { "score": 0, "cost": 0 },
                { "score": -1, "cost": -1 },
                { "score": -2, "cost": -3 },
                { "score": -3, "cost": -6 }
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn characteristic_display_matches_serde_scalar() {
        for c in Characteristic::ALL {
            let scalar = serde_json::to_value(c).unwrap();
            assert_eq!(scalar.as_str().unwrap(), c.to_string());
        }
    }

    #[test]
    fn cost_for_known_and_unknown_scores() {
        let r = rules();
        assert_eq!(r.cost_for(3), Some(6));
        assert_eq!(r.cost_for(0), Some(0));
        assert_eq!(r.cost_for(-1), Some(-1));
        assert_eq!(r.cost_for(-3), Some(-6));
        assert_eq!(r.cost_for(4), None);
        assert_eq!(r.cost_for(-4), None);
    }

    #[test]
    fn total_cost_nets_gains_against_spends() {
        let r = rules();
        // Darius's example (2358): Int +3 (6), Per +1 (1), Pre -3 (-6), Com -1
        // (-1), Sta 0 (0), Qik +2 (3), Str +2 (3), Dex +1 (1) => total 7.
        let scores = BTreeMap::from([
            (Characteristic::Int, 3),
            (Characteristic::Per, 1),
            (Characteristic::Pre, -3),
            (Characteristic::Com, -1),
            (Characteristic::Sta, 0),
            (Characteristic::Qik, 2),
            (Characteristic::Str, 2),
            (Characteristic::Dex, 1),
        ]);
        assert_eq!(r.total_cost(&scores), 7);
    }

    #[test]
    fn range_is_derived_from_table() {
        let r = rules();
        assert_eq!(r.min_score(), Some(-3));
        assert_eq!(r.max_score(), Some(3));
        assert!(r.is_legal_score(3));
        assert!(!r.is_legal_score(4));
    }

    #[test]
    fn rules_roundtrip() {
        let r = rules();
        let json = serde_json::to_string(&r).unwrap();
        let back: CharacteristicRules = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
        assert_eq!(r.start_points, 7);
    }
}
