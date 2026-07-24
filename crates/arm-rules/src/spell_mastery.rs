//! Spell Mastery special abilities: the catalogue of choosable per-spell mastery
//! options a magus may take for a mastered spell.
//!
//! For every level in a spell's Mastery Ability, the maga may choose one special
//! ability, applying only to that mastered spell (Core Rules.md:9524-9526). The
//! per-spell chosen abilities live on the [`SpellSelection`](crate::types::SpellSelection);
//! the catalogue here is language-neutral mechanics, with names/descriptions in
//! `rules/i18n`, keyed by `id`.
//!
//! Most abilities may be taken only once per spell; a few — Precise, Quick, and
//! Quiet Casting — may be taken multiple times for the same spell, flagged by
//! [`SpellMasteryAbility::repeatable`].
//!
//! Source: Ars Magica - Definitive Edition (Core Rules).md:9524-9592.

use serde::{Deserialize, Serialize};

use crate::types::{Id, SourceRef};

/// A single choosable Spell Mastery special ability in the catalogue. Its display
/// name and description live in `rules/i18n`, keyed by `id`.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:9528-9592.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellMasteryAbility {
    /// Slug-style id, e.g. `spell_mastery_ability.penetration`.
    pub id: Id,
    /// Whether this ability may be taken multiple times for the same spell.
    /// `true` only for Precise, Quick, and Quiet Casting
    /// (Core Rules.md:9572, :9576, :9580); `false` (the default) for the rest,
    /// which are once-per-spell.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub repeatable: bool,
    /// Provenance into the Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

/// The on-disk shape of `rules/core/spell_mastery_abilities.json`. Internal
/// deserialize-only wrapper (`pub(crate)`): parsed by [`crate::ruleset`] at load,
/// never part of the crate's public API. Matches the sibling arts/houses/spells
/// file wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct SpellMasteryAbilitiesFile {
    /// The Spell Mastery special ability catalogue.
    #[serde(default)]
    pub abilities: Vec<SpellMasteryAbility>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn ability_roundtrips_with_repeatable_flag() {
        let json = r#"{
          "id": "spell_mastery_ability.quiet_casting",
          "repeatable": true,
          "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [9578, 9580] }
        }"#;
        let ability: SpellMasteryAbility = serde_json::from_str(json).unwrap();
        assert_eq!(ability.id, Id::new("spell_mastery_ability.quiet_casting"));
        assert!(ability.repeatable);
        let back = serde_json::to_string(&ability).unwrap();
        assert_eq!(
            serde_json::from_str::<SpellMasteryAbility>(&back).unwrap(),
            ability
        );
    }

    /// A once-per-spell ability omits `repeatable` (default false, skip-if-false).
    #[test]
    fn non_repeatable_ability_omits_flag() {
        let ability = SpellMasteryAbility {
            id: Id::new("spell_mastery_ability.penetration"),
            repeatable: false,
            source: None,
        };
        let out = serde_json::to_string(&ability).unwrap();
        assert!(
            !out.contains("repeatable"),
            "default-false repeatable skipped: {out}"
        );
    }

    #[test]
    fn file_loads_catalogue() {
        let json = r#"{
          "abilities": [
            { "id": "spell_mastery_ability.penetration" },
            { "id": "spell_mastery_ability.precise_casting", "repeatable": true }
          ]
        }"#;
        let file: SpellMasteryAbilitiesFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.abilities.len(), 2);
        assert!(!file.abilities[0].repeatable);
        assert!(file.abilities[1].repeatable);
    }
}
