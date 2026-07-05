//! Hermetic Arts: the catalogue of Techniques and Forms, and the experience-point
//! advancement curve that converts XP into whole Art scores.
//!
//! Arts mirror [`Ability`](crate::ability::Ability): an [`Entity`] stores whole
//! bought Art scores plus one Art-XP bank, and the *effective* score (bought plus
//! virtue bonuses such as Puissant Art) is computed at validation time, never
//! stored. The one mechanical difference from Abilities is the price curve — Arts
//! use the cheaper triangular "ART To Buy" column (1, 3, 6, 10, 15, …), loaded as
//! its own [`AdvancementTable`] from `rules/core/arts.json`.
//!
//! The two Art classes (Technique, Form) are a fixed taxonomy and so an enum;
//! individual Arts and the advancement table are data.
//!
//! Source: Ars Magica - Definitive Edition (Core Rules).md:8833-8982 (the Arts
//! chapter), :2406-2427 (the "ART To Buy" advancement column).
//!
//! [`Entity`]: crate::types::Entity

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::ability::AdvancementTable;
use crate::types::{Id, SourceRef};

/// The two classes of Hermetic Art.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:8845-8897 (Techniques),
/// :8899-8982 (Forms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtType {
    /// Techniques — the verbs of magic (Creo, Intellego, Muto, Perdo, Rego).
    Technique,
    /// Forms — the nouns of magic (Animal, Aquam, … Vim).
    Form,
}

impl ArtType {
    /// Both classes in canonical book order (Techniques before Forms). The single
    /// source the serialized type-ordering is derived from, so the UI never
    /// re-hardcodes the order.
    pub const ALL: [ArtType; 2] = [ArtType::Technique, ArtType::Form];
}

impl fmt::Display for ArtType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ArtType::Technique => "technique",
            ArtType::Form => "form",
        })
    }
}

/// A single Art in the catalogue. Its display name and two-letter abbreviation
/// live in `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Art {
    /// Slug-style id, e.g. `art.creo`.
    pub id: Id,
    /// Whether this Art is a Technique or a Form.
    pub art_type: ArtType,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/arts.json`: the Art-XP advancement table and
/// the Art catalogue, mirroring the abilities file. Internal deserialize-only
/// wrapper (`pub(crate)`): parsed by [`crate::ruleset`] at load, never part of
/// the crate's public API — consumers see the assembled [`crate::Ruleset`], not
/// the raw file shapes. Matches the visibility of the sibling abilities/houses
/// file wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ArtsFile {
    /// The "ART To Buy" advancement table (triangular costs).
    #[serde(default)]
    pub advancement: AdvancementTable,
    /// The Art catalogue (5 Techniques + 10 Forms in the Core Rules).
    #[serde(default)]
    pub arts: Vec<Art>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn art_type_display_matches_serde_scalar() {
        for t in ArtType::ALL {
            let scalar = serde_json::to_value(t).unwrap();
            assert_eq!(scalar.as_str().unwrap(), t.to_string());
        }
    }

    #[test]
    fn techniques_sort_before_forms() {
        assert_eq!(ArtType::ALL, [ArtType::Technique, ArtType::Form]);
        assert!(ArtType::Technique < ArtType::Form);
    }

    #[test]
    fn art_roundtrip() {
        let json = r#"{
          "id": "art.creo",
          "art_type": "technique",
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [8847, 8863] }
        }"#;
        let art: Art = serde_json::from_str(json).unwrap();
        assert_eq!(art.id, Id::new("art.creo"));
        assert_eq!(art.art_type, ArtType::Technique);
        let back = serde_json::to_string(&art).unwrap();
        assert_eq!(serde_json::from_str::<Art>(&back).unwrap(), art);
    }

    /// The canonical ArM5 "ART To Buy" column is triangular n·(n+1)/2.
    #[test]
    fn arts_file_loads_triangular_advancement_and_catalogue() {
        let json = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 },
            { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 },
            { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        let file: ArtsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.advancement.xp_for_score(5), Some(15));
        assert_eq!(file.advancement.xp_to_raise(5), Some(5)); // 15 - 10
        assert_eq!(file.arts.len(), 2);
        assert_eq!(file.arts[0].id, Id::new("art.creo"));
        assert_eq!(file.arts[1].art_type, ArtType::Form);
    }
}
