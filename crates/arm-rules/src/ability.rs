//! Abilities: the catalogue of learned skills and the experience-point
//! advancement table that converts XP into whole Ability scores.
//!
//! The five Ability categories are a fixed enum. Individual abilities and the
//! XP advancement table are data, loaded into the [`Ruleset`] from
//! `rules/core/abilities.json`.
//!
//! Ability XP is spent in whole points — there is no partial progress *on* an
//! ability; loose XP sits in a character's bank until it can buy the next whole
//! point. So an [`Entity`] stores whole Ability scores plus one XP bank, and the
//! advancement table here is used to price each whole-point step ([`xp_to_raise`])
//! and to account for total XP committed ([`xp_for_score`]).
//!
//! [`Ruleset`]: crate::ruleset::Ruleset
//! [`Entity`]: crate::types::Entity
//! [`xp_to_raise`]: AdvancementTable::xp_to_raise
//! [`xp_for_score`]: AdvancementTable::xp_for_score

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

use crate::types::{Id, SourceRef};

/// The five Ability categories.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:7177-7268 (the Ability
/// list, grouped into General, Academic, Arcane, Martial, and Supernatural).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityCategory {
    /// General Abilities — anyone may learn them.
    General,
    /// Academic Abilities — require a Virtue (e.g. Educated) to learn.
    Academic,
    /// Arcane Abilities — require a Virtue to learn.
    Arcane,
    /// Martial Abilities — require a Virtue (e.g. Warrior) to learn.
    Martial,
    /// Supernatural Abilities — require a supernatural Virtue to learn.
    Supernatural,
}

impl fmt::Display for AbilityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AbilityCategory::General => "general",
            AbilityCategory::Academic => "academic",
            AbilityCategory::Arcane => "arcane",
            AbilityCategory::Martial => "martial",
            AbilityCategory::Supernatural => "supernatural",
        })
    }
}

/// A single Ability in the catalogue. Its display name lives in `rules/i18n`,
/// keyed by `id`; a character's chosen specialty is free text and is not part of
/// the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    /// Slug-style id, e.g. `ability.awareness`.
    pub id: Id,
    /// Which of the five categories this Ability belongs to.
    pub category: AbilityCategory,
    /// For a parameterized ability (e.g. `(Area) Lore`, `(Living Language)`), the
    /// key of the player-supplied parameter — `area`, `language`, … This key names
    /// the `{key}` placeholder in the localized name template and the
    /// `param-label-<key>` Fluent label. `None` for plain abilities, which then
    /// allow only one instance per character.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// One row of the Ability advancement table: `total_xp` is the experience needed
/// to reach `score` from zero.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2406-2427
/// ("ABILITY To Buy" column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityXpRow {
    /// The Ability score this row prices.
    pub score: u8,
    /// Total XP required to reach `score` from zero.
    pub total_xp: u32,
}

/// The Ability XP advancement table, loaded as data.
///
/// # JSON shape
///
/// `#[serde(transparent)]` over a `Vec`: the serialized form is a **bare JSON
/// array** of [`AbilityXpRow`] (NOT an object wrapping a `rows` field), e.g.
/// `[ { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 } ]`. This
/// bare-array shape is a stable public contract the frontend binds to.
///
/// Rows are kept sorted ascending by `score`, regardless of input order, so the
/// serialized array is canonical (project rule: arrays sorted by id/score). Both
/// [`AdvancementTable::new`] and deserialization enforce this; the lookups
/// ([`xp_for_score`], [`xp_to_raise`]) search by `score` and are unaffected by
/// ordering.
///
/// [`xp_for_score`]: AdvancementTable::xp_for_score
/// [`xp_to_raise`]: AdvancementTable::xp_to_raise
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(transparent)]
pub struct AdvancementTable {
    rows: Vec<AbilityXpRow>,
}

impl<'de> Deserialize<'de> for AdvancementTable {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let rows = Vec::<AbilityXpRow>::deserialize(deserializer)?;
        Ok(Self::new(rows))
    }
}

impl AdvancementTable {
    /// Builds a table from its rows, sorting them ascending by `score` so the
    /// serialized form is canonical regardless of input order.
    pub fn new(mut rows: Vec<AbilityXpRow>) -> Self {
        rows.sort_by_key(|row| row.score);
        Self { rows }
    }

    /// The rows, sorted ascending by `score`.
    pub fn rows(&self) -> &[AbilityXpRow] {
        &self.rows
    }

    /// The highest score the table prices (the last row, since rows are sorted
    /// ascending). `None` when the table is empty. Score 0 is always free and is
    /// not represented by a row, so an empty table still prices score 0.
    pub fn max_score(&self) -> Option<u8> {
        self.rows.last().map(|row| row.score)
    }

    /// Total XP committed to reach `score` from zero. Score 0 costs 0 XP; a score
    /// with no table row returns `None`.
    pub fn xp_for_score(&self, score: u8) -> Option<u32> {
        if score == 0 {
            return Some(0);
        }
        self.rows
            .iter()
            .find(|row| row.score == score)
            .map(|row| row.total_xp)
    }

    /// The XP cost of the single whole-point step from `score - 1` to `score`
    /// (the rulebook's "To Raise" value). Returns `None` for score 0 or a score
    /// the table does not cover.
    pub fn xp_to_raise(&self, score: u8) -> Option<u32> {
        if score == 0 {
            return None;
        }
        let to = self.xp_for_score(score)?;
        let from = self.xp_for_score(score - 1)?;
        Some(to - from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// The canonical ArM5 "ABILITY To Buy" column, scores 1-10 (Core Rules
    /// 2408-2417): triangular 5·n·(n+1)/2.
    fn table() -> AdvancementTable {
        serde_json::from_str(
            r#"[
              { "score": 1, "total_xp": 5 },
              { "score": 2, "total_xp": 15 },
              { "score": 3, "total_xp": 30 },
              { "score": 4, "total_xp": 50 },
              { "score": 5, "total_xp": 75 },
              { "score": 6, "total_xp": 105 },
              { "score": 7, "total_xp": 140 },
              { "score": 8, "total_xp": 180 },
              { "score": 9, "total_xp": 225 },
              { "score": 10, "total_xp": 275 }
            ]"#,
        )
        .unwrap()
    }

    #[test]
    fn category_display_matches_serde_scalar() {
        for c in [
            AbilityCategory::General,
            AbilityCategory::Academic,
            AbilityCategory::Arcane,
            AbilityCategory::Martial,
            AbilityCategory::Supernatural,
        ] {
            let scalar = serde_json::to_value(c).unwrap();
            assert_eq!(scalar.as_str().unwrap(), c.to_string());
        }
    }

    #[test]
    fn ability_roundtrip() {
        let json = r#"{
          "id": "ability.awareness",
          "category": "general",
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [7200, 7201] }
        }"#;
        let ability: Ability = serde_json::from_str(json).unwrap();
        assert_eq!(ability.id, Id::new("ability.awareness"));
        assert_eq!(ability.category, AbilityCategory::General);
        let back = serde_json::to_string(&ability).unwrap();
        assert_eq!(serde_json::from_str::<Ability>(&back).unwrap(), ability);
    }

    #[test]
    fn xp_for_score_is_triangular() {
        let t = table();
        assert_eq!(t.xp_for_score(0), Some(0));
        assert_eq!(t.xp_for_score(1), Some(5));
        assert_eq!(t.xp_for_score(5), Some(75));
        assert_eq!(t.xp_for_score(10), Some(275));
        assert_eq!(t.xp_for_score(11), None);
    }

    #[test]
    fn rows_serialize_score_sorted_regardless_of_input_order() {
        // Out-of-order input rows must serialize in ascending-score order.
        let t: AdvancementTable = serde_json::from_str(
            r#"[
              { "score": 3, "total_xp": 30 },
              { "score": 1, "total_xp": 5 },
              { "score": 2, "total_xp": 15 }
            ]"#,
        )
        .unwrap();
        let scores: Vec<u8> = t.rows().iter().map(|r| r.score).collect();
        assert_eq!(scores, vec![1, 2, 3]);

        // And via the constructor, lookups are unaffected.
        let built = AdvancementTable::new(t.rows().to_vec());
        assert_eq!(built.xp_for_score(2), Some(15));
        assert_eq!(built.xp_to_raise(3), Some(15)); // 30 - 15

        let json = serde_json::to_string(&t).unwrap();
        assert!(
            json.find("\"score\":1").unwrap() < json.find("\"score\":2").unwrap()
                && json.find("\"score\":2").unwrap() < json.find("\"score\":3").unwrap(),
            "serialized rows must be score-sorted: {json}"
        );
    }

    #[test]
    fn new_sorts_out_of_order_rows_and_lookups_are_correct() {
        // Build directly via the constructor (not serde) with deliberately
        // out-of-order rows; the constructor sorts ascending by score and the
        // lookups read the right values regardless of input order.
        let t = AdvancementTable::new(vec![
            AbilityXpRow {
                score: 3,
                total_xp: 30,
            },
            AbilityXpRow {
                score: 1,
                total_xp: 5,
            },
            AbilityXpRow {
                score: 2,
                total_xp: 15,
            },
        ]);

        let scores: Vec<u8> = t.rows().iter().map(|r| r.score).collect();
        assert_eq!(scores, vec![1, 2, 3], "rows() must be score-sorted");

        assert_eq!(t.xp_for_score(0), Some(0));
        assert_eq!(t.xp_for_score(2), Some(15));
        assert_eq!(t.xp_for_score(3), Some(30));
        assert_eq!(t.xp_for_score(4), None);

        assert_eq!(t.xp_to_raise(1), Some(5)); // 5 - 0
        assert_eq!(t.xp_to_raise(3), Some(15)); // 30 - 15
        assert_eq!(t.xp_to_raise(0), None);
    }

    #[test]
    fn xp_to_raise_is_the_increment() {
        let t = table();
        assert_eq!(t.xp_to_raise(0), None);
        assert_eq!(t.xp_to_raise(1), Some(5)); // 5 - 0
        assert_eq!(t.xp_to_raise(2), Some(10)); // 15 - 5
        assert_eq!(t.xp_to_raise(5), Some(25)); // 75 - 50
        assert_eq!(t.xp_to_raise(10), Some(50)); // 275 - 225
        assert_eq!(t.xp_to_raise(11), None);
    }
}
