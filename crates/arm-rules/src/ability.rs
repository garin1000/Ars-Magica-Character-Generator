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

use serde::{Deserialize, Serialize};
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

/// The Ability XP advancement table, loaded as data. Serializes transparently as
/// the JSON array of rows.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AdvancementTable {
    rows: Vec<AbilityXpRow>,
}

impl AdvancementTable {
    /// Builds a table from its rows.
    pub fn new(rows: Vec<AbilityXpRow>) -> Self {
        Self { rows }
    }

    /// The rows, in source order.
    pub fn rows(&self) -> &[AbilityXpRow] {
        &self.rows
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
