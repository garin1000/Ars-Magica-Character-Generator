//! Characteristics: the eight inborn attributes every character has, and the
//! point-buy cost table that governs them at character creation.
//!
//! The eight Characteristics are a fixed enum (the rules name exactly these
//! eight). The point-buy *cost table* is data, loaded into the [`Ruleset`] from
//! `rules/core/characteristics.json` — it is never hardcoded here, so the rule
//! numbers live with the other mechanics.
//!
//! [`Ruleset`]: crate::ruleset::Ruleset

use serde::{Deserialize, Deserializer, Serialize};
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

    /// The slug-style id for this Characteristic, e.g. `characteristic.str`.
    /// Used as a parameter value in the `characteristic` parameter domain.
    pub fn id(self) -> crate::types::Id {
        crate::types::Id::new(format!("characteristic.{self}"))
    }

    /// Parses a `characteristic.<slug>` [`Id`](crate::types::Id) back into a
    /// Characteristic, or `None` if it is not a valid characteristic id.
    pub fn from_id(id: &crate::types::Id) -> Option<Self> {
        let slug = id.as_str().strip_prefix("characteristic.")?;
        Self::ALL.into_iter().find(|c| c.to_string() == slug)
    }
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
/// The `costs` vector is kept sorted ascending by `score`, regardless of input
/// order, so the serialized array is canonical (project rule: arrays sorted by
/// id/score). Deserialization enforces this; the lookups search by `score` and
/// are unaffected by ordering.
///
/// Two distinct limits bound a base score: the *base* limit, the highest/lowest
/// score buyable with no limit-shifting virtue/flaw (the rulebook's "+3"), and
/// the *effective* limit, the absolute ceiling/floor reachable once Great or
/// Poor (Characteristic) shift it (the "+5" / "−5"). The cost table itself spans
/// the effective range so those scores can be priced.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2340-2354 (table),
/// :4105 (the "+3 unless Great Characteristic" base cap), :3987-3989 (Great's
/// "+5"), :6598-6600 (Poor's "−5").
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharacteristicRules {
    /// Points available to spend at creation (the rulebook's "seven points").
    pub start_points: u8,
    /// The cost table, one row per legal score, sorted ascending by `score`.
    pub costs: Vec<CharacteristicCost>,
    /// The highest base score buyable with no limit-shifting virtue (the "+3"
    /// cap). `None` falls back to the table maximum. Source: Core Rules.md:4105.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_max: Option<i8>,
    /// The lowest base score buyable with no limit-shifting flaw (the "−3"
    /// floor). `None` falls back to the table minimum. Source: Core
    /// Rules.md:6598-6600.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_min: Option<i8>,
    /// The highest base score reachable once Great (Characteristic) raises the
    /// cap (the "+5" ceiling). `None` falls back to the table maximum. Source:
    /// Core Rules.md:3987-3989.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_max: Option<i8>,
    /// The lowest base score reachable once Poor (Characteristic) lowers the
    /// floor (the "−5" floor). `None` falls back to the table minimum. Source:
    /// Core Rules.md:6598-6600.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_min: Option<i8>,
}

impl<'de> Deserialize<'de> for CharacteristicRules {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            start_points: u8,
            costs: Vec<CharacteristicCost>,
            #[serde(default)]
            base_max: Option<i8>,
            #[serde(default)]
            base_min: Option<i8>,
            #[serde(default)]
            effective_max: Option<i8>,
            #[serde(default)]
            effective_min: Option<i8>,
        }
        let Raw {
            start_points,
            mut costs,
            base_max,
            base_min,
            effective_max,
            effective_min,
        } = Raw::deserialize(deserializer)?;
        costs.sort_by_key(|row| row.score);
        Ok(Self {
            start_points,
            costs,
            base_max,
            base_min,
            effective_max,
            effective_min,
        })
    }
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

    /// The highest base score buyable with no limit-shifting virtue: the
    /// [`base_max`](Self::base_max) cap when present, else the table maximum.
    /// `None` only when the table itself is empty.
    pub fn base_max_score(&self) -> Option<i8> {
        self.base_max.or_else(|| self.max_score())
    }

    /// The lowest base score buyable with no limit-shifting flaw: the
    /// [`base_min`](Self::base_min) floor when present, else the table minimum.
    /// `None` only when the table itself is empty.
    pub fn base_min_score(&self) -> Option<i8> {
        self.base_min.or_else(|| self.min_score())
    }

    /// The highest base score reachable once Great (Characteristic) raises the
    /// cap: the [`effective_max`](Self::effective_max) ceiling when present, else
    /// the table maximum. `None` only when the table itself is empty.
    pub fn effective_max_score(&self) -> Option<i8> {
        self.effective_max.or_else(|| self.max_score())
    }

    /// The lowest base score reachable once Poor (Characteristic) lowers the
    /// floor: the [`effective_min`](Self::effective_min) floor when present, else
    /// the table minimum. `None` only when the table itself is empty.
    pub fn effective_min_score(&self) -> Option<i8> {
        self.effective_min.or_else(|| self.min_score())
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

    /// The shipped table once Great/Poor (Characteristic) extend it to ±5, with
    /// the base cap/floor at ±3 and the effective ceiling/floor at ±5.
    fn rules_with_great_poor() -> CharacteristicRules {
        serde_json::from_str(
            r#"{
              "start_points": 7,
              "base_max": 3, "base_min": -3,
              "effective_max": 5, "effective_min": -5,
              "costs": [
                { "score": 5, "cost": 15 },
                { "score": 4, "cost": 10 },
                { "score": 3, "cost": 6 },
                { "score": 2, "cost": 3 },
                { "score": 1, "cost": 1 },
                { "score": 0, "cost": 0 },
                { "score": -1, "cost": -1 },
                { "score": -2, "cost": -3 },
                { "score": -3, "cost": -6 },
                { "score": -4, "cost": -10 },
                { "score": -5, "cost": -15 }
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn cost_for_extended_table_prices_great_and_poor_scores() {
        let r = rules_with_great_poor();
        assert_eq!(r.cost_for(4), Some(10));
        assert_eq!(r.cost_for(5), Some(15));
        assert_eq!(r.cost_for(-4), Some(-10));
        assert_eq!(r.cost_for(-5), Some(-15));
        // +6 / -6 are still off the table.
        assert_eq!(r.cost_for(6), None);
        assert_eq!(r.cost_for(-6), None);
    }

    #[test]
    fn base_limits_distinguish_from_effective_limits() {
        let r = rules_with_great_poor();
        // Base cap/floor are the no-virtue ±3 limits...
        assert_eq!(r.base_max_score(), Some(3));
        assert_eq!(r.base_min_score(), Some(-3));
        // ...while the effective limits are the Great/Poor ±5 ceilings...
        assert_eq!(r.effective_max_score(), Some(5));
        assert_eq!(r.effective_min_score(), Some(-5));
        // ...and the table itself spans the full effective range.
        assert_eq!(r.max_score(), Some(5));
        assert_eq!(r.min_score(), Some(-5));
    }

    #[test]
    fn limits_fall_back_to_table_bounds_when_absent() {
        // The legacy 7-row table omits every explicit limit; all four accessors
        // then fall back to the table bounds (±3).
        let r = rules();
        assert_eq!(r.base_max_score(), Some(3));
        assert_eq!(r.base_min_score(), Some(-3));
        assert_eq!(r.effective_max_score(), Some(3));
        assert_eq!(r.effective_min_score(), Some(-3));
    }

    #[test]
    fn effective_max_overrides_table_max_when_present() {
        // Without an explicit effective_max, the table maximum is the ceiling.
        assert_eq!(rules().effective_max_score(), Some(3));
        // With one (Great Characteristic's +5), it raises the ceiling.
        let r: CharacteristicRules = serde_json::from_str(
            r#"{ "start_points": 7, "effective_max": 5,
                 "costs": [{ "score": 3, "cost": 6 }, { "score": 0, "cost": 0 }] }"#,
        )
        .unwrap();
        assert_eq!(r.effective_max_score(), Some(5));
        assert_eq!(r.max_score(), Some(3));
    }

    #[test]
    fn characteristic_id_roundtrips() {
        for c in Characteristic::ALL {
            assert_eq!(Characteristic::from_id(&c.id()), Some(c));
        }
        assert_eq!(
            Characteristic::from_id(&crate::types::Id::new("characteristic.str")),
            Some(Characteristic::Str)
        );
        assert_eq!(
            Characteristic::from_id(&crate::types::Id::new("characteristic.nope")),
            None
        );
        // A non-characteristic namespace is rejected.
        assert_eq!(
            Characteristic::from_id(&crate::types::Id::new("ability.awareness")),
            None
        );
    }

    #[test]
    fn costs_serialize_score_sorted_regardless_of_input_order() {
        // The canonical table above is authored high-to-low; after load the
        // costs must be ascending by score, and serialization stays canonical.
        let r = rules();
        let scores: Vec<i8> = r.costs.iter().map(|c| c.score).collect();
        assert_eq!(scores, vec![-3, -2, -1, 0, 1, 2, 3]);
        // Lookups are unaffected by the re-sort.
        assert_eq!(r.cost_for(3), Some(6));
        assert_eq!(r.cost_for(-3), Some(-6));

        let json = serde_json::to_string(&r).unwrap();
        let first = json.find("\"score\":-3").unwrap();
        let last = json.find("\"score\":3").unwrap();
        assert!(
            first < last,
            "serialized costs must be score-sorted: {json}"
        );
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
