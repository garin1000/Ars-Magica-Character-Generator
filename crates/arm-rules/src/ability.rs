//! Abilities: the catalogue of learned skills and the experience-point
//! advancement table that converts XP into whole Ability scores.
//!
//! The five Ability categories are a fixed enum. Individual abilities, the XP
//! advancement table, and the age → maximum-Ability-score band table are data,
//! loaded into the [`Ruleset`] from `rules/core/abilities.json`.
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

use crate::types::{Id, SourceRef, is_false};

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

impl AbilityCategory {
    /// All categories in canonical book order. The single source the serialized
    /// category-ordering is derived from, so the UI never re-hardcodes the order.
    pub const ALL: [AbilityCategory; 5] = [
        AbilityCategory::General,
        AbilityCategory::Academic,
        AbilityCategory::Arcane,
        AbilityCategory::Martial,
        AbilityCategory::Supernatural,
    ];
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
    /// Whether this Ability is "asterisked" in the rulebook: it cannot be used
    /// without at least one experience point in it — there is no untrained roll.
    /// This is an independent per-ability property, **not** derived from
    /// `category`: it is true for ~50 Core abilities spanning General (e.g.
    /// (Area) Lore, Chirurgy), Academic, Arcane, and all Supernatural abilities.
    /// The UI renders a trailing `*` after the name when this is set.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:4157 (the Jack of
    /// All Trades Virtue spells out the rule: "Characters without this Virtue
    /// cannot even attempt rolls on an asterisked Ability without at least one
    /// experience point in it."). Each asterisked ability's `####` heading in the
    /// Abilities chapter (`:7273-7786`) carries the `*` that sets this flag.
    #[serde(default, skip_serializing_if = "is_false")]
    pub requires_training: bool,
    /// Whether this Ability may be used as a weapon's combat Ability. True for
    /// the Martial Abilities and — as a data flag rather than a hardcoded slug —
    /// for Brawl, the General Ability used for unarmed and improvised weapons and
    /// for dodging without a Martial Ability. Drives the weapon load-time trust
    /// gate ([`Ruleset::validate_weapon_refs`](crate::ruleset::Ruleset)) so a
    /// ruleset that slugs unarmed combat differently just sets the flag rather
    /// than mis-rejecting.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:7337-7340 (Brawl:
    /// "Fighting hand-to-hand without weapons, or with the sorts of improvised
    /// weapons you just pick up … also the Ability used to dodge attacks if you
    /// have no Martial Abilities.").
    #[serde(default, skip_serializing_if = "is_false")]
    pub combat_ability: bool,
    /// Whether this Ability is tied to where the character grew up, so a Flaw that
    /// narrows locality knowledge caps it lower at creation
    /// ([`Effect::LocalityAbilityCapFraction`](crate::types::Effect)).
    ///
    /// A per-ability flag rather than a category or a hardcoded slug list, because
    /// the rules name three families and then trail off: "locality-dependent
    /// Abilities like Language, Area Lore, or Organization Lore, **as well as some
    /// social Abilities**". The three named families carry the flag; which social
    /// Abilities a saga counts is left to the data author rather than guessed here.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:6160 (Foreign
    /// Upbringing).
    #[serde(default, skip_serializing_if = "is_false")]
    pub locality_dependent: bool,
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

    /// The highest score whose cumulative `total_xp` is at or below `xp` — the
    /// inverse of [`xp_for_score`]. Score 0 is always free, so an `xp` below the
    /// first row yields 0; an `xp` at or above the top row yields the top score
    /// (the curve is clamped, never extrapolated). Decrepitude and Warping rise
    /// "like an Ability" up this same table (Core:16464-16475, :16617), so this is
    /// how a bank of accrued points becomes a score.
    ///
    /// Rows are score-sorted ascending at construction, so the last row not
    /// exceeding `xp` is the answer.
    ///
    /// [`xp_for_score`]: AdvancementTable::xp_for_score
    pub fn score_for_xp(&self, xp: u32) -> u8 {
        self.rows
            .iter()
            .filter(|row| row.total_xp <= xp)
            .map(|row| row.score)
            .max()
            .unwrap_or(0)
    }

    /// The XP cost of the single whole-point step from `score - 1` to `score`
    /// (the rulebook's "To Raise" value). Returns `None` for score 0 or a score
    /// the table does not cover.
    ///
    /// Load-time validation ([`validation_errors`]) guarantees `total_xp` is
    /// non-decreasing, so the subtraction never underflows for a table that came
    /// through [`Ruleset::from_sources`]. `checked_sub` is belt-and-braces: a
    /// table built outside that gate degrades to `None` instead of panicking.
    ///
    /// [`validation_errors`]: AdvancementTable::validation_errors
    /// [`Ruleset::from_sources`]: crate::ruleset::Ruleset::from_sources
    pub fn xp_to_raise(&self, score: u8) -> Option<u32> {
        if score == 0 {
            return None;
        }
        let to = self.xp_for_score(score)?;
        let from = self.xp_for_score(score - 1)?;
        to.checked_sub(from)
    }

    /// Data-integrity check on the table itself, returning one message per
    /// violation (empty when valid). The table is valid iff scores are unique and
    /// `total_xp` is non-decreasing as `score` increases — the precondition that
    /// makes [`xp_to_raise`]'s step subtraction sound. Called from
    /// [`Ruleset::validate_integrity`] so malformed data fails loudly at load
    /// rather than underflow-panicking on first use.
    ///
    /// Rows are score-sorted at construction, so a single forward pass suffices.
    ///
    /// [`xp_to_raise`]: AdvancementTable::xp_to_raise
    /// [`Ruleset::validate_integrity`]: crate::ruleset::Ruleset::validate_integrity
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for pair in self.rows.windows(2) {
            let (prev, curr) = (&pair[0], &pair[1]);
            if curr.score == prev.score {
                errors.push(format!(
                    "advancement table: duplicate score {} ({} and {} total_xp)",
                    curr.score, prev.total_xp, curr.total_xp
                ));
            } else if curr.total_xp < prev.total_xp {
                errors.push(format!(
                    "advancement table: total_xp decreases from score {} ({}) to score {} ({})",
                    prev.score, prev.total_xp, curr.score, curr.total_xp
                ));
            }
        }
        errors
    }
}

/// One band of the age → maximum-Ability-score table: a character aged `max_age`
/// or younger caps every Ability at `max_score` at character creation. The final,
/// open-ended band omits `max_age` and covers every older character.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2366-2374.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgeAbilityCap {
    /// Inclusive upper age bound of this band; `None` for the open-ended band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    /// The maximum Ability score for a character in this band.
    pub max_score: u8,
}

/// The age → maximum-Ability-score band table, loaded as data.
///
/// # JSON shape
///
/// `#[serde(transparent)]` over a `Vec`: the serialized form is a **bare JSON
/// array** of [`AgeAbilityCap`] (NOT an object), e.g.
/// `[ { "max_age": 29, "max_score": 5 }, { "max_score": 9 } ]`.
///
/// Bands are kept sorted ascending by age with the open-ended band (no `max_age`)
/// last, regardless of input order, so the serialized array is canonical (project
/// rule: arrays sorted by id/score). Both [`AgeAbilityCaps::new`] and
/// deserialization enforce this; [`max_ability_score`] walks this order and takes
/// the first covering band.
///
/// This table lives beside the Ability advancement table because it caps *Ability*
/// scores by age (the Core Rules "Age | Maximum Ability" table), not any
/// Characteristic.
///
/// [`max_ability_score`]: AgeAbilityCaps::max_ability_score
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(transparent)]
pub struct AgeAbilityCaps {
    bands: Vec<AgeAbilityCap>,
}

impl<'de> Deserialize<'de> for AgeAbilityCaps {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bands = Vec::<AgeAbilityCap>::deserialize(deserializer)?;
        Ok(Self::new(bands))
    }
}

impl AgeAbilityCaps {
    /// Builds the table from its bands, sorting them ascending by age with the
    /// open-ended band (no `max_age`) last, so the serialized form is canonical
    /// regardless of input order.
    pub fn new(mut bands: Vec<AgeAbilityCap>) -> Self {
        bands.sort_by_key(|band| (band.max_age.unwrap_or(u32::MAX), band.max_score));
        Self { bands }
    }

    /// The bands, in canonical order (age-ascending, open-ended band last).
    pub fn bands(&self) -> &[AgeAbilityCap] {
        &self.bands
    }

    /// `true` when the ruleset ships no age bands (the cap is then unknowable and
    /// not enforced).
    pub fn is_empty(&self) -> bool {
        self.bands.is_empty()
    }

    /// The maximum Ability score a character of `age` may buy at creation, per the
    /// age band table (Core:2366-2374), or `None` if the ruleset ships no age caps
    /// (the cap is then unknowable and not enforced). The bands are age-ascending
    /// with the open-ended band last, so the first band whose `max_age` covers
    /// `age` — or the open-ended band — gives the cap. Some Virtues raise this
    /// limit; that is applied by the caller (e.g. Affinity's +2 in validation).
    pub fn max_ability_score(&self, age: u32) -> Option<u8> {
        self.bands.iter().find_map(|band| match band.max_age {
            Some(max) if age <= max => Some(band.max_score),
            None => Some(band.max_score),
            _ => None,
        })
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
        // Absent `requires_training` defaults to false (Awareness is usable untrained).
        assert!(!ability.requires_training);
        let back = serde_json::to_string(&ability).unwrap();
        assert_eq!(serde_json::from_str::<Ability>(&back).unwrap(), ability);
        // A non-asterisked ability omits the flag from canonical JSON.
        assert!(!back.contains("requires_training"));
    }

    #[test]
    fn requires_training_roundtrips_and_is_independent_of_category() {
        // Artes Liberales is Academic and asterisked; the flag is its own property.
        let json = r#"{
          "id": "ability.artes_liberales",
          "category": "academic",
          "requires_training": true,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [7307, 7320] }
        }"#;
        let ability: Ability = serde_json::from_str(json).unwrap();
        assert!(ability.requires_training);
        assert_eq!(ability.category, AbilityCategory::Academic);
        let back = serde_json::to_string(&ability).unwrap();
        assert!(back.contains("\"requires_training\":true"));
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
    fn score_for_xp_inverts_the_curve() {
        // The inverse of `xp_for_score`: the highest score whose cumulative XP is
        // ≤ the given total. Decrepitude/Warping rise "like an Ability" up this
        // same table, so this is how accrued points become a score.
        let t = table();
        assert_eq!(t.score_for_xp(0), 0);
        assert_eq!(t.score_for_xp(4), 0); // below the first row (5)
        assert_eq!(t.score_for_xp(5), 1); // exactly score 1
        assert_eq!(t.score_for_xp(14), 1); // between 5 and 15
        assert_eq!(t.score_for_xp(15), 2); // exactly score 2 (warping worked value)
        assert_eq!(t.score_for_xp(17), 2); // 17 aging points → Decrepitude 2 (Core:16617)
        assert_eq!(t.score_for_xp(30), 3);
        assert_eq!(t.score_for_xp(275), 10);
        assert_eq!(t.score_for_xp(10_000), 10); // clamps at the table's top row
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

    #[test]
    fn canonical_table_has_no_validation_errors() {
        assert!(table().validation_errors().is_empty());
    }

    #[test]
    fn non_monotonic_total_xp_is_an_error() {
        // total_xp drops from score 2 (15) to score 3 (10): xp_to_raise(3) would
        // underflow, so this must be reported at validation time.
        let t = AdvancementTable::new(vec![
            AbilityXpRow {
                score: 1,
                total_xp: 5,
            },
            AbilityXpRow {
                score: 2,
                total_xp: 15,
            },
            AbilityXpRow {
                score: 3,
                total_xp: 10,
            },
        ]);
        let errors = t.validation_errors();
        assert!(
            errors.iter().any(|m| m.contains("total_xp decreases")
                && m.contains("score 2")
                && m.contains("score 3")),
            "expected a decreasing-total_xp error naming scores 2 and 3, got {errors:?}"
        );
    }

    #[test]
    fn duplicate_score_is_an_error() {
        let t = AdvancementTable::new(vec![
            AbilityXpRow {
                score: 1,
                total_xp: 5,
            },
            AbilityXpRow {
                score: 1,
                total_xp: 5,
            },
        ]);
        let errors = t.validation_errors();
        assert!(
            errors
                .iter()
                .any(|m| m.contains("duplicate score") && m.contains('1')),
            "expected a duplicate-score error naming score 1, got {errors:?}"
        );
    }

    /// The shipped age → max-Ability-score bands (Core:2366-2374), authored out of
    /// order so the sort-on-load is exercised.
    fn age_caps() -> AgeAbilityCaps {
        serde_json::from_str(
            r#"[
              { "max_age": 45, "max_score": 8 },
              { "max_age": 29, "max_score": 5 },
              { "max_score": 9 },
              { "max_age": 35, "max_score": 6 },
              { "max_age": 40, "max_score": 7 }
            ]"#,
        )
        .unwrap()
    }

    #[test]
    fn age_ability_cap_reads_the_band_covering_the_age() {
        let caps = age_caps();
        assert_eq!(caps.max_ability_score(0), Some(5));
        assert_eq!(caps.max_ability_score(29), Some(5));
        assert_eq!(caps.max_ability_score(30), Some(6));
        assert_eq!(caps.max_ability_score(35), Some(6));
        assert_eq!(caps.max_ability_score(36), Some(7));
        assert_eq!(caps.max_ability_score(40), Some(7));
        assert_eq!(caps.max_ability_score(41), Some(8));
        assert_eq!(caps.max_ability_score(45), Some(8));
        assert_eq!(caps.max_ability_score(46), Some(9));
        assert_eq!(caps.max_ability_score(200), Some(9));
    }

    #[test]
    fn age_ability_caps_sort_ascending_by_age_open_band_last() {
        // Authored out of order above; after load the bands are age-ascending with
        // the open-ended (no max_age) band last, so the array is canonical.
        let caps = age_caps();
        let ages: Vec<Option<u32>> = caps.bands().iter().map(|b| b.max_age).collect();
        assert_eq!(ages, vec![Some(29), Some(35), Some(40), Some(45), None]);
    }

    #[test]
    fn age_ability_cap_absent_table_yields_none() {
        // A ruleset that ships no age bands; the cap is then unknowable.
        let caps = AgeAbilityCaps::default();
        assert!(caps.is_empty());
        assert_eq!(caps.max_ability_score(25), None);
    }
}
