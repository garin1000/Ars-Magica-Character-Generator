//! Hermetic spells: the catalogue of formulaic/ritual spells a magus may know at
//! character creation.
//!
//! A spell is defined by its Technique + Form (both [`Art`](crate::art::Art) ids),
//! its Level, and any Art requisites. Level is either a fixed multiple of five or
//! **General** — a General spell is learned at a per-character chosen level, and
//! two General versions at different levels are different spells
//! (Core Rules.md:12349-12353). The character's chosen spells and (for General
//! spells) their learned levels live on the [`Entity`](crate::types::Entity) as
//! [`SpellSelection`](crate::types::SpellSelection)s; the catalogue here is
//! language-neutral mechanics, with names/descriptions in `rules/i18n`.
//!
//! Spells consume the magus's *spell-levels budget* (120 at creation,
//! Core Rules.md:2215-2216, 2435), a currency distinct from the shared Ability/Art
//! XP pool. Enforcement lives in [`crate::validation`].

use serde::{Deserialize, Serialize};

use crate::types::{Id, SourceRef};

/// A single spell in the catalogue. Its display name and description live in
/// `rules/i18n`, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    /// Slug-style id, e.g. `spell.pilum_of_fire`.
    pub id: Id,
    /// The spell's Technique (a Technique-class Art id, e.g. `art.creo`).
    pub technique: Id,
    /// The spell's Form (a Form-class Art id, e.g. `art.ignem`).
    pub form: Id,
    /// Fixed level (`Some`, a multiple of five) or **General** (`None`), where a
    /// General spell is learned at a per-character chosen level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// Art requisites, if any (each a catalogue Art id). Display detail for M4;
    /// not factored into the per-spell level cap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requisites: Vec<Id>,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/spells.json`. Internal deserialize-only
/// wrapper (`pub(crate)`): parsed by [`crate::ruleset`] at load, never part of the
/// crate's public API. Matches the sibling arts/houses file wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct SpellsFile {
    /// The spell catalogue.
    #[serde(default)]
    pub spells: Vec<Spell>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fixed_spell_roundtrips() {
        let json = r#"{
          "id": "spell.pilum_of_fire",
          "technique": "art.creo",
          "form": "art.ignem",
          "level": 20,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [14250, 14253] }
        }"#;
        let spell: Spell = serde_json::from_str(json).unwrap();
        assert_eq!(spell.id, Id::new("spell.pilum_of_fire"));
        assert_eq!(spell.technique, Id::new("art.creo"));
        assert_eq!(spell.form, Id::new("art.ignem"));
        assert_eq!(spell.level, Some(20));
        assert!(spell.requisites.is_empty());
        let back = serde_json::to_string(&spell).unwrap();
        assert_eq!(serde_json::from_str::<Spell>(&back).unwrap(), spell);
    }

    /// A General spell omits `level`; a spell may carry requisites.
    #[test]
    fn general_and_requisite_spells_load() {
        let json = r#"{
          "spells": [
            { "id": "spell.aegis_of_the_hearth", "technique": "art.rego", "form": "art.vim" },
            { "id": "spell.coat_of_flame", "technique": "art.creo", "form": "art.ignem",
              "level": 25, "requisites": ["art.rego"] }
          ]
        }"#;
        let file: SpellsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.spells.len(), 2);
        assert_eq!(file.spells[0].level, None); // General
        assert_eq!(file.spells[1].requisites, vec![Id::new("art.rego")]);
    }

    /// A General spell serializes without a `level` key (skip-if-none).
    #[test]
    fn general_spell_omits_level_key() {
        let spell = Spell {
            id: Id::new("spell.aegis_of_the_hearth"),
            technique: Id::new("art.rego"),
            form: Id::new("art.vim"),
            level: None,
            requisites: Vec::new(),
            source: None,
        };
        let out = serde_json::to_string(&spell).unwrap();
        assert!(
            !out.contains("level"),
            "General spell must omit level: {out}"
        );
        assert!(
            !out.contains("requisites"),
            "empty requisites skipped: {out}"
        );
    }
}
